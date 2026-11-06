//! Self-development end-to-end integration test.
//!
//! Proves the full self-improvement cycle:
//!   CodebaseFacts → analyze → Proposal → owner approve → WorkUnitSpec
//!   → schedule → dispatch → result → verify → record_completion → persist
//!
//! This is the "Deep Create is alive" gate from the execution contract:
//! the system can propose a change to itself, get owner authorization,
//! execute it through the real pipeline, verify the outcome, and persist
//! the completed state.

use dc_core::self_development::SelfDevelopmentRuntime;
use dc_core::{
    ContextPlan, CoreCommand, CoreDispatcher, CorePersistence, CoreState, PersistedCoreState,
    SchedulingContext, VerificationOutcome, compile_prompt_bundle, handle_command,
    run_machine_verification, select_ready_units,
};
use dc_domain::ids::{AuthorityEpoch, EntityId, RowVersion, RunGeneration};
use dc_domain::permission::{
    ActionKind, CanonicalResource, CapabilityLease, DataClass, LeaseExpiry, LeaseScope, LeaseStatus,
};
use dc_domain::resource::ExplicitLimit;
use dc_domain::self_development::CodebaseFacts;
use dc_domain::work::{
    DependencyKey, DependencyKind, DependencySnapshot, GraphCandidate, ObjectiveContract,
    UnitStage, VersionedDependency, WorkKind, WorkScope,
};
use dc_store::MemoryStore;
use dc_worker::WorkDispatcher;
use std::collections::BTreeSet;
use std::num::NonZeroU32;

// ============================================================================
// Test infrastructure (mirrors end_to_end.rs conventions)
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

/// Stub dispatcher that simulates a worker producing a patch proposal.
struct SelfDevDispatcher;

impl WorkDispatcher for SelfDevDispatcher {
    fn dispatch(
        &self,
        request: &dc_worker::ExecuteRequest,
    ) -> Result<dc_worker::ExecuteResult, dc_worker::WorkerError> {
        Ok(dc_worker::ExecuteResult::Success(dc_worker::WorkResponse {
            content: format!(
                "self-dev patch for unit {}: fix applied to target area",
                request.execution_id
            ),
            model: "self-dev-model".to_string(),
            tokens_used: Some(128),
            finish_reason: Some("stop".to_string()),
        }))
    }
}

// ============================================================================
// Self-development end-to-end test
// ============================================================================

#[test]
fn self_development_e2e() {
    // ── Phase 1: Self-analysis ──────────────────────────────────────────
    let mut self_dev = SelfDevelopmentRuntime::new();
    let mut id_counter = 1000u128;
    let mut id_allocator = || {
        id_counter += 1;
        test_id(id_counter)
    };

    // Supply codebase facts that trigger a proposal: area without integration tests
    let facts = CodebaseFacts {
        test_coverage: vec![dc_domain::self_development::TestCoverageFact {
            area: "auth-timestamp-boundary".to_string(),
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

    assert!(
        !proposals.is_empty(),
        "analyze must produce at least one proposal for uncovered area"
    );
    let proposal = proposals[0].clone();

    assert!(
        SelfDevelopmentRuntime::requires_authorization(&proposal),
        "self-dev proposals must require owner authorization"
    );

    // ── Phase 2: Owner approval ─────────────────────────────────────────
    let objective = ObjectiveContract {
        scope: scope.clone(),
        statement: "Improve test coverage for security-critical paths".to_string(),
        owner_event_ids: vec![],
        obligations: vec![],
        completion_criteria: vec![test_id(100)],
        row_version: RowVersion::new(1).unwrap(),
    };
    let unit_spec = self_dev
        .approve_proposal(proposal.proposal_id, &objective, &mut id_allocator)
        .expect("proposal approval must succeed");
    assert_eq!(unit_spec.kind, WorkKind::LocalChangeProposal);
    assert_eq!(unit_spec.stage, UnitStage::Proposed);

    // ── Phase 3: Core pipeline — accept, schedule, dispatch ─────────────
    let store = MemoryStore::new();
    let mut persistence = CorePersistence::new(store);
    persistence
        .initialize(std::path::Path::new(":memory:"))
        .unwrap();

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
    assert!(state.has_active_work());

    // Schedule
    let sched_ctx = SchedulingContext {
        leases: vec![test_lease()],
        max_concurrent: 5,
        running_attempts: 0,
    };
    let ready = select_ready_units(&graph_clone, &sched_ctx).unwrap();
    assert_eq!(ready.ready.len(), 1);
    let unit_ref = ready.ready[0];
    handle_command(&mut state, CoreCommand::StartAttempt(unit_ref)).unwrap();

    // Compile context
    let context_plan = ContextPlan {
        memory_scope: vec![],
        allowed_data_classes: vec![DataClass::Public],
        max_context_tokens: 4096,
    };
    let lease = test_lease();
    let context = compile_prompt_bundle(&unit_spec, &context_plan, &lease).unwrap();
    assert!(!context.system_prompt.is_empty());

    // Dispatch
    let dispatcher = CoreDispatcher::new(SelfDevDispatcher, "self-dev-worker".to_string());
    let result = dispatcher
        .execute_unit(&unit_spec, &context.system_prompt, &context.user_prompt)
        .unwrap();
    let response = match result {
        dc_worker::ExecuteResult::Success(r) => r,
        _ => panic!("expected success from self-dev dispatcher"),
    };
    assert!(response.content.contains("self-dev patch"));

    // ── Phase 4: Record result + verify ─────────────────────────────────
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
        CoreCommand::RecordResult {
            unit: unit_ref,
            result: result_packet,
        },
    )
    .unwrap();

    // Machine verification
    let verification_outcome =
        run_machine_verification(&unit_spec, &result_packet_clone, test_id(600)).unwrap();
    let verification_record = match &verification_outcome {
        VerificationOutcome::Passed { record } => record.clone(),
        _ => panic!("expected verification to pass for self-dev unit"),
    };

    handle_command(
        &mut state,
        CoreCommand::RecordVerification(verification_record),
    )
    .unwrap();
    handle_command(&mut state, CoreCommand::ApplyVerifiedResult(unit_ref)).unwrap();

    // ── Phase 5: Record completion in self-dev runtime ──────────────────
    let now_millis = 1_727_000_000_000u64; // 2026-09-25 approx
    self_dev.record_completion(
        unit_spec.unit.unit_id,
        proposal.proposal_id,
        verification_outcome,
        now_millis,
    );
    assert_eq!(self_dev.completed_units().len(), 1);
    assert_eq!(
        self_dev.approved_units().len(),
        0,
        "unit must be removed from approved after completion"
    );
    assert_eq!(
        self_dev.pending_proposals().len(),
        0,
        "proposal must be consumed after approval"
    );

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

    println!("✅ Self-development e2e PASSED: proposal → approve → dispatch → verify → persist");
}
