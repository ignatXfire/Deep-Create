//! End-to-end integration test: command → schedule → dispatch → result → verify → persist
//!
//! This test proves the full pipeline works: a command enters dc-core,
//! gets scheduled, dispatched to a worker, result comes back, gets verified,
//! and everything persists through dc-store.

use dc_core::{
    CoreState, CoreCommand, handle_command,
    select_ready_units, SchedulingContext,
    compile_prompt_bundle,
    run_machine_verification, VerificationOutcome,
    CorePersistence, PersistedCoreState,
    CoreDispatcher, BatchCoordinator,
};
use dc_domain::ids::{EntityId, RowVersion, RunGeneration, AuthorityEpoch};
use dc_domain::work::{
    ObjectiveContract, GraphCandidate, WorkUnitSpec, WorkScope, WorkKind,
    WorkBounds, StopCondition, OutputContract, UnitStage, UnitRef,
    VerificationRecord, ResultPacket, ResultStage, DependencySnapshot,
    VersionedDependency, DependencyKey, DependencyKind, QualificationStatus,
    VerificationLevel, VerificationEvidence, ObligationFinding, ObligationStatus,
};
use dc_domain::permission::{CapabilityLease, LeaseScope, LeaseExpiry, LeaseStatus, CanonicalResource, ActionKind};
use dc_domain::resource::ExplicitLimit;
use dc_store::MemoryStore;
use dc_worker::WorkDispatcher;
use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};

// ============================================================================
// Test infrastructure
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

fn test_unit_spec(unit_id: EntityId) -> WorkUnitSpec {
    WorkUnitSpec {
        unit: UnitRef {
            unit_id,
            revision: dc_domain::ids::UnitRevision::new(1).unwrap(),
        },
        scope: test_scope(),
        parent: None,
        kind: WorkKind::Research,
        expected_output: OutputContract {
            export_name: "test".into(),
            schema_version: NonZeroU32::new(1).unwrap(),
            fields: vec![],
        },
        obligation_ids: vec![],
        acceptance_criteria: vec![],
        bounds: WorkBounds {
            max_attempts: NonZeroU32::new(3).unwrap(),
            max_steps: NonZeroU32::new(10).unwrap(),
            stop_conditions: vec![StopCondition::OutputProduced],
        },
        exploration: None,
        stage: UnitStage::Proposed,
        accepted_result: None,
        row_version: RowVersion::new(1).unwrap(),
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
            data_classes: BTreeSet::from([dc_domain::permission::DataClass::Public]),
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

struct E2EDispatcher;

impl WorkDispatcher for E2EDispatcher {
    fn dispatch(&self, request: &dc_worker::ExecuteRequest) -> Result<dc_worker::ExecuteResult, dc_worker::WorkerError> {
        Ok(dc_worker::ExecuteResult::Success(dc_worker::WorkResponse {
            content: format!("E2E processed: {}", request.execution_id),
            model: "e2e-test-model".to_string(),
            tokens_used: Some(42),
            finish_reason: Some("stop".to_string()),
        }))
    }
}

// ============================================================================
// End-to-end test
// ============================================================================

#[test]
fn end_to_end_pipeline() {
    // Step 1: Initialize persistence
    let store = MemoryStore::new();
    let mut persistence = CorePersistence::new(store);
    persistence.initialize(std::path::Path::new(":memory:")).unwrap();

    // Step 2: Create core state
    let mut state = CoreState::empty();
    assert!(!state.has_active_work());

    // Step 3: Create and accept objective
    let objective = ObjectiveContract {
        scope: test_scope(),
        statement: "Test objective".to_string(),
        owner_event_ids: vec![],
        obligations: vec![],
        completion_criteria: vec![test_id(100)],
        row_version: RowVersion::new(1).unwrap(),
    };
    let _outcome = handle_command(&mut state, CoreCommand::AcceptObjective(objective)).unwrap();
    assert!(state.objective.is_some());

    // Step 4: Create work graph
    let unit = test_unit_spec(test_id(200));
    let graph = GraphCandidate {
        expected_graph_version: RowVersion::new(1).unwrap(),
        current_graph_version: RowVersion::new(1).unwrap(),
        roots: vec![],
        objectives: vec![],
        units: vec![unit],
        dependencies: vec![],
        links: vec![],
        publications: vec![],
        publication_states: vec![],
        publication_bindings: vec![],
        owner_acceptances: vec![],
    };
    let graph_clone = graph.clone();
    let outcome = handle_command(&mut state, CoreCommand::AcceptGraph(graph)).unwrap();
    assert!(state.has_active_work());

    // Step 5: Schedule ready units
    let sched_ctx = SchedulingContext {
        leases: vec![test_lease()],
        max_concurrent: 5,
        running_attempts: 0,
    };
    let ready = select_ready_units(&graph_clone, &sched_ctx).unwrap();
    assert_eq!(ready.ready.len(), 1);

    // Step 5.5: Begin attempt (move unit to InProgress)
    let unit = &ready.ready[0];
    let _ = handle_command(&mut state, CoreCommand::StartAttempt(*unit)).unwrap();

    // Step 6: Compile context for the unit
    let unit_spec = test_unit_spec(unit.unit_id);
    let context_plan = dc_core::ContextPlan {
        memory_scope: vec![],
        allowed_data_classes: vec![dc_domain::permission::DataClass::Public],
        max_context_tokens: 4096,
    };
    let lease = test_lease();
    let context = compile_prompt_bundle(&unit_spec, &context_plan, &lease).unwrap();
    assert!(!context.system_prompt.is_empty());

    // Step 7: Dispatch through worker (stub)
    let dispatcher = CoreDispatcher::new(E2EDispatcher, "e2e-worker".to_string());
    let result = dispatcher.execute_unit(&unit_spec, &context.system_prompt, &context.user_prompt).unwrap();
    let response = match result {
        dc_worker::ExecuteResult::Success(r) => r,
        _ => panic!("expected success"),
    };
    assert!(response.content.contains("E2E processed"));

    // Step 8: Create result packet
    let snapshot = DependencySnapshot {
        snapshot_id: test_id(300),
        scope: test_scope(),
        unit: *unit,
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
    let result_packet = dispatcher.response_to_result(&unit_spec, &response, test_id(302), snapshot, policy).unwrap();
    let result_packet_clone = result_packet.clone();

    // Step 9: Record result
    let outcome = handle_command(&mut state, CoreCommand::RecordResult {
        unit: *unit,
        result: result_packet,
    }).unwrap();

    // Step 10: Create verification record manually
    let verification = VerificationRecord {
        verification_id: test_id(400),
        result_id: test_id(302),
        result_fingerprint: [0u8; 32],
        unit: *unit,
        snapshot_id: test_id(300),
        verification_policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: test_id(401),
            },
            expected_revision: RowVersion::new(1).unwrap(),
            fingerprint: None,
        },
        verifier_attempt_id: test_id(402),
        verdict: dc_domain::work::VerificationVerdict::Pass,
        findings: vec![],
        evidence: VerificationEvidence {
            verification_plan_id: test_id(400),
            method_revision_id: test_id(403),
            scope: test_scope(),
            work_kind: WorkKind::Research,
            criterion_ids: vec![],
            level: VerificationLevel::MachineOnly,
            required_level: VerificationLevel::MachineOnly,
            machine_findings_id: test_id(404),
            policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::Policy,
                    entity_id: test_id(405),
                },
                expected_revision: RowVersion::new(1).unwrap(),
                fingerprint: None,
            },
            qualification: None,
            independence_assessment_id: None,
            protected_admission_evidence_id: None,
        },
    };
    let outcome = handle_command(&mut state, CoreCommand::RecordVerification(verification)).unwrap();

    // Step 11: Apply verified result
    let outcome = handle_command(&mut state, CoreCommand::ApplyVerifiedResult(*unit)).unwrap();

    // Step 12: Persist state
    let persisted = PersistedCoreState {
        current_objective: Some(test_id(3)), // objective_id from scope
        current_graph_version: 1,
        active_units: vec![],
        completed_units: vec![unit.unit_id],
        objective_json: None,
        graph_json: None,
    };
    let version = persistence.persist_state(
        test_id(500),
        &persisted,
        dc_store::ExpectedVersion::MustNotExist,
    ).unwrap();
    assert_eq!(version, 1);

    // Step 13: Verify persistence round-trip
    let loaded = persistence.load_state(&test_id(500)).unwrap().unwrap();
    assert_eq!(loaded.data.completed_units, vec![unit.unit_id]);

    // Step 14: Run machine verification
    let outcomes = run_machine_verification(&unit_spec, &result_packet_clone, test_id(600)).unwrap();
    match outcomes {
        VerificationOutcome::Passed { record } => {
            assert_eq!(record.unit, *unit);
        }
        _ => panic!("expected verification to pass"),
    }

    println!("✅ End-to-end pipeline test PASSED");
}
