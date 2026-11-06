//! Live self-development end-to-end test.
//!
//! Same full cycle as `self_development_e2e.rs` but with a REAL dispatcher:
//!   CodebaseFacts → analyze → Proposal → owner approve → WorkUnitSpec
//!   → schedule → CoreDispatcher(BrokerDispatcher) → HTTP → live OmniRoute
//!   → result → verify → record_completion → persist
//!
//! Auto-skips when no endpoint is listening on 127.0.0.1:20128.

use dc_broker::dispatch::{dispatch_work_unit_sync, DispatchRequest, DispatchResult};
use dc_broker::endpoint::{EndpointConfig, ModelEndpoint};
use dc_core::{
    CoreState, CoreCommand, handle_command,
    select_ready_units, SchedulingContext,
    compile_prompt_bundle, ContextPlan,
    run_machine_verification, VerificationOutcome,
    CorePersistence, PersistedCoreState,
    CoreDispatcher,
};
use dc_core::self_development::SelfDevelopmentRuntime;
use dc_domain::ids::{EntityId, RowVersion, RunGeneration, AuthorityEpoch};
use dc_domain::self_development::CodebaseFacts;
use dc_domain::work::{
    ObjectiveContract, GraphCandidate, WorkScope, WorkKind,
    DependencySnapshot, UnitStage,
    VersionedDependency, DependencyKey, DependencyKind,
};
use dc_domain::permission::{CapabilityLease, LeaseScope, LeaseExpiry, LeaseStatus, CanonicalResource, ActionKind, DataClass};
use dc_domain::resource::ExplicitLimit;
use dc_store::MemoryStore;
use dc_worker::{ExecuteRequest, ExecuteResult, WorkDispatcher, WorkerError, WorkResponse};
use std::collections::BTreeSet;
use std::num::NonZeroU32;

// ============================================================================
// Infrastructure (same fixtures as self_development_e2e.rs)
// ============================================================================

fn test_id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n)
        .expect("synthetic RFC 9562 UUIDv7")
}

fn test_scope() -> WorkScope {
    WorkScope {
        session_id: test_id(1),
        work_id: test_id(2),
        objective_id: test_id(3),
        objective_revision: NonZeroU32::new(1).unwrap(),
    }
}

fn test_lease() -> CapabilityLease {
    CapabilityLease {
        lease_id: test_id(900),
        owner_event_id: test_id(901),
        installation_id: test_id(902),
        scope: LeaseScope {
            work_id: test_id(903),
            unit_id: None,
            action_kind: ActionKind::CallModelConnection,
            resource: CanonicalResource::ModelConnection {
                connection_id: test_id(905),
                revision: RowVersion::new(1).unwrap(),
            },
            recipient: None,
            data_classes: BTreeSet::from([DataClass::Public]),
            max_calls: ExplicitLimit::Finite(100),
            max_bytes: ExplicitLimit::Finite(1_000_000),
            cost_limits: Default::default(),
        },
        not_before_utc_millis: 0,
        expiry: LeaseExpiry::Never,
        parent_lease_id: None,
        delegation_depth: 0,
        max_delegation_depth: 3,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        run_generation: RunGeneration::new(1).unwrap(),
        support_fences: vec![],
        confirmed_conditions_hash: [0u8; 32],
        issued_dependency_snapshot_id: test_id(904),
        lease_version: RowVersion::new(1).unwrap(),
        status: LeaseStatus::Active,
    }
}

/// REAL dispatcher: routes through dc-broker to live OmniRoute.
struct BrokerDispatcher {
    endpoint: ModelEndpoint,
}

impl WorkDispatcher for BrokerDispatcher {
    fn dispatch(&self, request: &ExecuteRequest) -> Result<ExecuteResult, WorkerError> {
        let broker_req = DispatchRequest {
            dispatch_id: request.execution_id.clone(),
            work_unit_spec: request.work_unit_spec.clone(),
            system_prompt: request.system_prompt.clone(),
            user_prompt: request.user_prompt.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };
        match dispatch_work_unit_sync(&self.endpoint, &broker_req) {
            Ok(DispatchResult::Success(resp)) => Ok(ExecuteResult::Success(WorkResponse {
                content: resp.content,
                model: resp.model,
                tokens_used: resp.tokens_used,
                finish_reason: resp.finish_reason,
            })),
            Ok(DispatchResult::Refused(reason)) => Ok(ExecuteResult::Refused(reason)),
            Ok(DispatchResult::Unreachable(reason)) => Ok(ExecuteResult::Unreachable(reason)),
            Err(e) => Err(WorkerError::Dispatch(format!("{}", e))),
        }
    }
}

// ============================================================================
// Live self-development end-to-end test
// ============================================================================

#[test]
fn live_self_development_e2e() {
    // Skip when no live endpoint (CI / offline).
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:20128".parse().unwrap(),
        std::time::Duration::from_secs(2),
    )
    .is_err()
    {
        eprintln!("SKIP: no live endpoint on 127.0.0.1:20128");
        return;
    }

    // ── Phase 1: Self-analysis ──────────────────────────────────────────
    let mut self_dev = SelfDevelopmentRuntime::new();
    let mut id_counter = 1000u128;
    let mut id_allocator = || {
        id_counter += 1;
        test_id(id_counter)
    };

    let facts = CodebaseFacts {
        test_coverage: vec![dc_domain::self_development::TestCoverageFact {
            area: "live-self-dev-boundary".to_string(),
            has_integration_tests: false,
            has_property_tests: false,
            evidence_id: test_id(600),
        }],
        documentation: vec![],
        protocol: vec![],
        security: vec![],
    };
    let scope = test_scope();
    let proposals = self_dev.analyze(&facts, &scope, &mut id_allocator);
    assert!(!proposals.is_empty(), "analyze must produce at least one proposal");
    let proposal = proposals[0].clone();
    assert!(SelfDevelopmentRuntime::requires_authorization(&proposal));

    // ── Phase 2: Owner approval ─────────────────────────────────────────
    let objective = ObjectiveContract {
        scope: scope.clone(),
        statement: "Live self-dev: improve test coverage for security-critical paths".to_string(),
        owner_event_ids: vec![],
        obligations: vec![],
        completion_criteria: vec![test_id(100)],
        row_version: RowVersion::new(1).unwrap(),
    };
    let unit_spec = self_dev
        .approve_proposal(proposal.proposal_id, &objective, &mut id_allocator)
        .expect("proposal approval must succeed");
    assert_eq!(unit_spec.kind, WorkKind::LocalChangeProposal);

    // ── Phase 3: Core pipeline with REAL dispatcher ─────────────────────
    let store = MemoryStore::new();
    let mut persistence = CorePersistence::new(store);
    persistence.initialize(std::path::Path::new(":memory:")).unwrap();

    let mut state = CoreState::empty();
    handle_command(&mut state, CoreCommand::AcceptObjective(objective.clone())).unwrap();

    let graph = GraphCandidate {
        expected_graph_version: RowVersion::new(1).unwrap(),
        current_graph_version: RowVersion::new(1).unwrap(),
        roots: vec![],
        objectives: vec![],
        units: vec![unit_spec.clone()],
        dependencies: vec![],
        links: vec![],
        publications: vec![],
        publication_states: vec![],
        publication_bindings: vec![],
        owner_acceptances: vec![],
    };
    let graph_clone = graph.clone();
    handle_command(&mut state, CoreCommand::AcceptGraph(graph)).unwrap();

    let sched_ctx = SchedulingContext {
        leases: vec![test_lease()],
        max_concurrent: 5,
        running_attempts: 0,
    };
    let ready = select_ready_units(&graph_clone, &sched_ctx).unwrap();
    assert_eq!(ready.ready.len(), 1);
    let unit_ref = ready.ready[0];
    handle_command(&mut state, CoreCommand::StartAttempt(unit_ref)).unwrap();

    let context_plan = ContextPlan {
        memory_scope: vec![],
        allowed_data_classes: vec![DataClass::Public],
        max_context_tokens: 4096,
    };
    let lease = test_lease();
    let context = compile_prompt_bundle(&unit_spec, &context_plan, &lease).unwrap();

    // REAL dispatch through dc-broker to live OmniRoute
    let mut cfg = EndpointConfig::default();
    cfg.max_retries = 0;
    cfg.timeout_ms = 90_000;
    let dispatcher = CoreDispatcher::new(
        BrokerDispatcher { endpoint: ModelEndpoint::new(cfg) },
        "live-self-dev-worker".to_string(),
    );
    let result = dispatcher
        .execute_unit(&unit_spec, &context.system_prompt, &context.user_prompt)
        .unwrap();
    let response = match result {
        ExecuteResult::Success(r) => r,
        ExecuteResult::Refused(r) => panic!("live model refused: {}", r),
        ExecuteResult::Unreachable(r) => panic!("live endpoint unreachable: {}", r),
    };

    // Live proof: real model name, token accounting, non-empty content
    eprintln!(
        "LIVE-SELF-DEV: model={} tokens={:?} finish={:?} content_len={}",
        response.model, response.tokens_used, response.finish_reason, response.content.len()
    );
    assert!(!response.model.is_empty(), "real model name must be present");
    assert!(response.tokens_used.is_some(), "token accounting must be present");

    // ── Phase 4: Record result + verify + persist ───────────────────────
    let snapshot = DependencySnapshot {
        snapshot_id: test_id(300),
        scope: scope.clone(),
        unit: unit_ref,
        read_set: vec![],
        policy_versions: vec![],
    };
    let policy = VersionedDependency {
        key: DependencyKey {
            kind: DependencyKind::VerificationCriteria,
            entity_id: test_id(301),
        },
        expected_revision: RowVersion::new(1).unwrap(),
        fingerprint: None,
    };
    let result_packet = dispatcher
        .response_to_result(&unit_spec, &response, test_id(302), snapshot, policy)
        .unwrap();
    let result_packet_clone = result_packet.clone();

    handle_command(
        &mut state,
        CoreCommand::RecordResult { unit: unit_ref, result: result_packet },
    )
    .unwrap();

    // Machine verification
    let verification_outcome = run_machine_verification(&unit_spec, &result_packet_clone, test_id(600)).unwrap();
    let verification_record = match &verification_outcome {
        VerificationOutcome::Passed { record } => record.clone(),
        _ => panic!("expected verification to pass for live self-dev unit"),
    };

    handle_command(
        &mut state,
        CoreCommand::RecordVerification(verification_record),
    )
    .unwrap();
    handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref)).unwrap();

    // ── Phase 5: Record completion in self-dev runtime ──────────────────
    let now_millis = 1_727_000_000_000u64;
    self_dev.record_completion(
        unit_spec.unit.unit_id,
        proposal.proposal_id,
        verification_outcome,
        now_millis,
    );
    assert_eq!(self_dev.completed_units().len(), 1);

    // ── Phase 6: Persist and verify round-trip ──────────────────────────
    let persisted = PersistedCoreState {
        current_objective: Some(test_id(3)),
        current_graph_version: 1,
        active_units: vec![],
        completed_units: vec![unit_spec.unit.unit_id],
        objective_json: None,
        graph_json: None,
    };
    let version = persistence
        .persist_state(
            test_id(500),
            &persisted,
            dc_store::ExpectedVersion::MustNotExist,
        )
        .unwrap();
    assert_eq!(version, 1);

    let loaded = persistence.load_state(&test_id(500)).unwrap().unwrap();
    assert_eq!(loaded.data.completed_units, vec![unit_spec.unit.unit_id]);

    eprintln!("LIVE-SELF-DEV OK: full cycle with real model dispatch completed");
}
