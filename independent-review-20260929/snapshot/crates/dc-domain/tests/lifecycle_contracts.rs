use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};

use dc_domain::ids::{EntityId, RowVersion};
use dc_domain::work::*;

#[path = "cases/owner_return.rs"]
mod owner_return;

#[path = "cases/continuation.rs"]
mod continuation;

fn id(v: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | v)
        .expect("synthetic RFC 9562 UUIDv7")
}
fn row(v: u64) -> RowVersion {
    NonZeroU64::new(v).unwrap()
}
fn rev(v: u32) -> NonZeroU32 {
    NonZeroU32::new(v).unwrap()
}
fn scope() -> WorkScope {
    WorkScope {
        session_id: id(1),
        work_id: id(2),
        objective_id: id(3),
        objective_revision: rev(1),
    }
}
fn unit() -> WorkUnitSpec {
    WorkUnitSpec {
        unit: UnitRef {
            unit_id: id(4),
            revision: rev(1),
        },
        scope: scope(),
        parent: None,
        kind: WorkKind::Research,
        expected_output: OutputContract {
            export_name: "report".into(),
            schema_version: rev(1),
            fields: vec![],
        },
        obligation_ids: vec![id(5)],
        acceptance_criteria: vec![id(5)],
        bounds: WorkBounds {
            max_attempts: rev(2),
            max_steps: rev(10),
            stop_conditions: vec![StopCondition::StepLimitReached],
        },
        exploration: None,
        stage: UnitStage::Ready,
        accepted_result: None,
        row_version: row(1),
    }
}
fn root() -> RootWork {
    RootWork {
        work_id: id(2),
        session_id: id(1),
        active_objective_id: id(3),
        stage: RootStage::Draft,
        blocking_reasons: BTreeSet::new(),
        row_version: row(1),
    }
}
fn test_objective() -> ObjectiveContract {
    ObjectiveContract {
        scope: scope(),
        statement: "Test objective".into(),
        owner_event_ids: vec![id(1)],
        obligations: vec![],
        completion_criteria: vec![id(5)],
        row_version: row(1),
    }
}
fn test_verification_record(vid: EntityId, unit: &WorkUnitSpec) -> VerificationRecord {
    VerificationRecord {
        verification_id: vid,
        result_id: id(1000),
        result_fingerprint: [0u8; 32],
        unit: unit.unit,
        snapshot_id: id(1001),
        verification_policy: VersionedDependency {
            key: DependencyKey {
                kind: DependencyKind::VerificationCriteria,
                entity_id: id(1002),
            },
            expected_revision: row(1),
            fingerprint: None,
        },
        verifier_attempt_id: id(1003),
        verdict: VerificationVerdict::Pass,
        findings: vec![],
        evidence: VerificationEvidence {
            verification_plan_id: id(1005),
            method_revision_id: id(1006),
            scope: unit.scope,
            work_kind: unit.kind,
            criterion_ids: unit.acceptance_criteria.clone(),
            level: VerificationLevel::IndependentReview,
            required_level: VerificationLevel::IndependentReview,
            machine_findings_id: id(1007),
            policy: VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::VerificationCriteria,
                    entity_id: id(1002),
                },
                expected_revision: row(1),
                fingerprint: None,
            },
            qualification: Some(MethodQualification {
                qualification_id: id(1004),
                method_revision_id: id(1006),
                work_kind: unit.kind,
                criterion_ids: unit.acceptance_criteria.clone(),
                level: VerificationLevel::IndependentReview,
                policy: VersionedDependency {
                    key: DependencyKey {
                        kind: DependencyKind::VerificationCriteria,
                        entity_id: id(1002),
                    },
                    expected_revision: row(1),
                    fingerprint: None,
                },
                status: QualificationStatus::Qualified,
            }),
            independence_assessment_id: None,
            protected_admission_evidence_id: None,
        },
    }
}
fn expected(state: &UnitAttemptState) -> ExpectedLifecycleBasis {
    ExpectedLifecycleBasis {
        scope: scope(),
        unit: state.unit.unit,
        unit_row_version: state.unit.row_version,
        attempt_id: state.attempt.as_ref().map(|a| a.attempt_id),
        attempt_row_version: state.attempt.as_ref().map(|a| a.row_version),
        run_generation: row(1),
    }
}
fn step(
    state: &UnitAttemptState,
    command: UnitAttemptCommand,
) -> Result<UnitLifecyclePlan, WorkError> {
    plan_unit_attempt_transition(
        state,
        &expected(state),
        scope(),
        row(1),
        &BTreeSet::new(),
        &command,
    )
}
fn started() -> UnitAttemptState {
    let initial = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    let leased = step(
        &initial,
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(10),
        },
    )
    .unwrap()
    .replacement()
    .unwrap()
    .clone();
    step(&leased, UnitAttemptCommand::Start {})
        .unwrap()
        .replacement()
        .unwrap()
        .clone()
}

const GATE_OPERATIONS: [GateOperation; 11] = [
    GateOperation::Read,
    GateOperation::Reason,
    GateOperation::Plan,
    GateOperation::Effect,
    GateOperation::Cancel,
    GateOperation::TrustedLocalDiagnosticRead,
    GateOperation::Reconcile,
    GateOperation::ObserveFact,
    GateOperation::ReceiveOwnerDecision,
    GateOperation::SubmitResult,
    GateOperation::AcceptCompletion,
];

fn operation_column(operation: GateOperation) -> usize {
    // Deliberately exhaustive and separate from the production policy.
    match operation {
        GateOperation::Read => 0,
        GateOperation::Reason => 1,
        GateOperation::Plan => 2,
        GateOperation::Effect => 3,
        GateOperation::Cancel => 4,
        GateOperation::TrustedLocalDiagnosticRead => 5,
        GateOperation::Reconcile => 6,
        GateOperation::ObserveFact => 7,
        GateOperation::ReceiveOwnerDecision => 8,
        GateOperation::SubmitResult => 9,
        GateOperation::AcceptCompletion => 10,
    }
}

fn gate_row(reason: &BlockingReason) -> usize {
    match reason {
        BlockingReason::Recovery {} => 0,
        BlockingReason::IntegrityFailure {} => 1,
        BlockingReason::ResourceUnavailable {} => 2,
        BlockingReason::StopRequested {} => 3,
        BlockingReason::UnknownOutcome {} => 4,
        BlockingReason::AwaitingOwner {} => 5,
        BlockingReason::AwaitingPublication { .. } => 6,
        BlockingReason::PublicationUnresolved { .. } => 7,
        BlockingReason::PublicationWithdrawn { .. } => 8,
        BlockingReason::OwnerAcceptancePending { .. } => 9,
        BlockingReason::ConditionUnknown { .. } => 10,
        BlockingReason::UpstreamCancelled { .. } => 11,
        BlockingReason::DependencyPending { .. } => 12,
        BlockingReason::OwnerRejectionPending { .. } => 13,
    }
}

fn gate_contract_rows() -> [(BlockingReason, [u8; 11]); 14] {
    // Literal oracle: R, reason, plan, effect, cancel, local diagnostic,
    // reconcile, fact, owner input, semantic submission, final acceptance.
    [
        (
            BlockingReason::Recovery {},
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::IntegrityFailure {},
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::ResourceUnavailable {},
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::StopRequested {},
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::UnknownOutcome {},
            [1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::AwaitingOwner {},
            [1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1],
        ),
        (
            BlockingReason::AwaitingPublication { link_id: id(80) },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::PublicationUnresolved {
                publication_id: id(81),
            },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::PublicationWithdrawn {
                publication_id: id(82),
            },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::OwnerAcceptancePending {
                publication_id: id(83),
            },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::ConditionUnknown { link_id: id(84) },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::UpstreamCancelled { unit: unit().unit },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::DependencyPending { unit: unit().unit },
            [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
        (
            BlockingReason::OwnerRejectionPending {
                owner_event_id: id(40),
            },
            [1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0],
        ),
    ]
}

#[test]
fn every_gate_operation_cell_matches_the_independent_contract_table() {
    let rows = gate_contract_rows();
    assert_eq!(
        GATE_OPERATIONS.into_iter().collect::<BTreeSet<_>>().len(),
        11
    );
    for (index, operation) in GATE_OPERATIONS.into_iter().enumerate() {
        assert_eq!(operation_column(operation), index);
    }
    for (index, (reason, expected)) in rows.iter().enumerate() {
        assert_eq!(gate_row(reason), index);
        for (column, operation) in GATE_OPERATIONS.into_iter().enumerate() {
            let actual = evaluate_gates([reason], &[operation]).unwrap();
            assert_eq!(
                actual.is_clear(),
                expected[column] == 1,
                "{reason:?} × {operation:?}"
            );
            assert_eq!(actual.blockers().contains(reason), expected[column] == 0);
        }
    }
}

#[test]
fn all_gate_combinations_are_conjunctive_for_every_operation_and_scope() {
    let rows = gate_contract_rows();
    let mut cells = 0;
    for mask in 0_u16..(1 << rows.len()) {
        let mut installation = BTreeSet::new();
        let mut root = BTreeSet::new();
        let mut unit = BTreeSet::new();
        for (index, (reason, _)) in rows.iter().enumerate() {
            if mask & (1 << index) != 0 {
                match index % 3 {
                    0 => installation.insert(reason.clone()),
                    1 => root.insert(reason.clone()),
                    _ => unit.insert(reason.clone()),
                };
            }
        }
        for (column, operation) in GATE_OPERATIONS.into_iter().enumerate() {
            let expected: BTreeSet<_> = rows
                .iter()
                .enumerate()
                .filter(|(index, (_, flags))| mask & (1 << index) != 0 && flags[column] == 0)
                .map(|(_, (reason, _))| reason.clone())
                .collect();
            let actual =
                evaluate_gates(installation.iter().chain(&root).chain(&unit), &[operation])
                    .unwrap();
            assert_eq!(
                actual.blockers(),
                &expected,
                "mask={mask:#06x}, operation={operation:?}"
            );
            let reversed =
                evaluate_gates(unit.iter().chain(&root).chain(&installation), &[operation])
                    .unwrap();
            assert_eq!(actual, reversed, "scope order cannot change restrictions");
            cells += 1;
        }
    }
    assert_eq!(cells, 180_224);
}

#[test]
fn compound_operations_cannot_hide_effects_or_upgrade_owner_input() {
    for (reason, flags) in gate_contract_rows() {
        for (left, a) in GATE_OPERATIONS.into_iter().enumerate() {
            for (right, b) in GATE_OPERATIONS.into_iter().enumerate() {
                let result = evaluate_gates([&reason], &[a, b]).unwrap();
                assert_eq!(result.is_clear(), flags[left] == 1 && flags[right] == 1);
            }
        }
    }
    assert_eq!(
        evaluate_gates(&BTreeSet::new(), &[]).unwrap_err(),
        WorkError::InvalidField("gate_operations"),
    );
    let waiting = BTreeSet::from([BlockingReason::AwaitingOwner {}]);
    assert!(
        evaluate_gates(&waiting, &[GateOperation::Plan])
            .unwrap()
            .is_clear()
    );
    assert!(
        !evaluate_gates(&waiting, &[GateOperation::Plan, GateOperation::Effect])
            .unwrap()
            .is_clear()
    );
    let recovery = BTreeSet::from([BlockingReason::Recovery {}]);
    assert!(
        evaluate_gates(&recovery, &[GateOperation::ReceiveOwnerDecision])
            .unwrap()
            .is_clear()
    );
    assert!(
        !evaluate_gates(
            &recovery,
            &[
                GateOperation::ReceiveOwnerDecision,
                GateOperation::AcceptCompletion
            ]
        )
        .unwrap()
        .is_clear()
    );
    assert!(
        evaluate_gates(&recovery, &[GateOperation::TrustedLocalDiagnosticRead])
            .unwrap()
            .is_clear()
    );
    assert!(
        !evaluate_gates(
            &recovery,
            &[
                GateOperation::TrustedLocalDiagnosticRead,
                GateOperation::Read
            ]
        )
        .unwrap()
        .is_clear()
    );
}

#[test]
fn clearing_one_reason_does_not_clear_another_or_promote_an_old_attempt() {
    let mut gates = BTreeSet::from([
        BlockingReason::Recovery {},
        BlockingReason::AwaitingOwner {},
        BlockingReason::UnknownOutcome {},
    ]);
    gates.remove(&BlockingReason::Recovery {});
    assert!(
        evaluate_gates(&gates, &[GateOperation::Plan])
            .unwrap()
            .is_clear()
    );
    assert_eq!(
        evaluate_gates(&gates, &[GateOperation::AcceptCompletion])
            .unwrap()
            .blockers(),
        &BTreeSet::from([BlockingReason::UnknownOutcome {}])
    );
    gates.remove(&BlockingReason::UnknownOutcome {});
    assert!(
        evaluate_gates(&gates, &[GateOperation::AcceptCompletion])
            .unwrap()
            .is_clear()
    );
    assert!(
        !evaluate_gates(&gates, &[GateOperation::Effect])
            .unwrap()
            .is_clear()
    );
    let state = started();
    let late = plan_unit_attempt_transition(
        &state,
        &expected(&state),
        scope(),
        row(2),
        &BTreeSet::new(),
        &UnitAttemptCommand::ObserveSubmission { result_id: id(90) },
    )
    .unwrap();
    assert!(
        late.observation_only(),
        "cleared gates do not restore an old run generation"
    );
}

#[test]
fn each_gate_blocks_unclassified_attempts_and_semantic_submission_not_protective_facts() {
    let initial = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    let prepared = step(
        &initial,
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(10),
        },
    )
    .unwrap()
    .replacement()
    .unwrap()
    .clone();
    let running = started();
    for (reason, _) in gate_contract_rows() {
        let gates = BTreeSet::from([reason.clone()]);
        for (state, command) in [
            (
                &initial,
                UnitAttemptCommand::Prepare {
                    new_attempt_id: id(11),
                },
            ),
            (&prepared, UnitAttemptCommand::Start {}),
        ] {
            assert_eq!(
                plan_unit_attempt_transition(
                    state,
                    &expected(state),
                    scope(),
                    row(1),
                    &gates,
                    &command
                )
                .unwrap_err(),
                WorkError::Blocked(reason.clone())
            );
        }
        let late = plan_unit_attempt_transition(
            &running,
            &expected(&running),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::ObserveSubmission { result_id: id(90) },
        )
        .unwrap();
        assert!(late.observation_only());
        assert_eq!(late.expected(), &expected(&running));
        assert_eq!(
            late.events(),
            &[LifecycleEvent::LateObservation {
                attempt_id: id(10),
                observation_id: id(90),
            }]
        );
        let cancelled = plan_unit_attempt_transition(
            &running,
            &expected(&running),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::RequestCancellation {},
        )
        .unwrap()
        .replacement()
        .unwrap()
        .clone();
        let confirmed = plan_unit_attempt_transition(
            &cancelled,
            &expected(&cancelled),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::ConfirmCancellation {},
        )
        .unwrap();
        assert_eq!(
            confirmed.replacement().unwrap().unit.stage,
            UnitStage::Cancelled
        );
        let lost = plan_unit_attempt_transition(
            &running,
            &expected(&running),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::ObserveLost {
                observation_id: id(91),
            },
        )
        .unwrap();
        assert_eq!(lost.replacement().unwrap().unit.stage, UnitStage::Lost);
        assert_eq!(gates, BTreeSet::from([reason]));
        assert_eq!(running.unit.stage, UnitStage::Running);
        assert!(running.unit.accepted_result.is_none());
    }
    let mut planning = initial;
    planning.unit.kind = WorkKind::Planning;
    assert!(
        matches!(
            plan_unit_attempt_transition(
                &planning,
                &expected(&planning),
                scope(),
                row(1),
                &BTreeSet::from([BlockingReason::AwaitingOwner {}]),
                &UnitAttemptCommand::Prepare {
                    new_attempt_id: id(11)
                }
            ),
            Err(WorkError::Blocked(_))
        ),
        "a Planning name is not an effect-free execution profile"
    );
}

#[test]
fn prepare_start_submit_is_one_exact_plan_per_step_not_acceptance() {
    let initial = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    assert!(step(&initial, UnitAttemptCommand::Start {}).is_err());
    let leased = step(
        &initial,
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(10),
        },
    )
    .unwrap();
    assert_eq!(leased.replacement().unwrap().unit.stage, UnitStage::Leased);
    assert_eq!(leased.replacement().unwrap().unit.row_version, row(2));
    assert_eq!(initial.unit.stage, UnitStage::Ready);
    let running = started();
    assert_eq!(
        running.attempt.as_ref().unwrap().stage,
        AttemptStage::Running
    );
    let submitted = step(
        &running,
        UnitAttemptCommand::ObserveSubmission { result_id: id(11) },
    )
    .unwrap();
    assert_eq!(
        submitted.replacement().unwrap().unit.stage,
        UnitStage::ResultSubmitted
    );
    assert_eq!(
        submitted
            .replacement()
            .unwrap()
            .attempt
            .as_ref()
            .unwrap()
            .stage,
        AttemptStage::Submitted
    );
    assert!(
        submitted
            .replacement()
            .unwrap()
            .unit
            .accepted_result
            .is_none()
    );
    assert_eq!(
        submitted.events(),
        &[LifecycleEvent::ResultSubmitted {
            attempt_id: id(10),
            result_id: id(11)
        }]
    );
}

#[test]
fn stale_version_objective_unit_attempt_and_run_are_rejected() {
    let state = started();
    let e = expected(&state);
    let mut bad = e.clone();
    bad.unit_row_version = row(1);
    assert_eq!(
        plan_unit_attempt_transition(
            &state,
            &bad,
            scope(),
            row(1),
            &BTreeSet::new(),
            &UnitAttemptCommand::RequestCancellation {}
        ),
        Err(WorkError::RowVersionConflict)
    );
    for bad in [
        ExpectedLifecycleBasis {
            scope: WorkScope {
                objective_revision: rev(2),
                ..scope()
            },
            ..e.clone()
        },
        ExpectedLifecycleBasis {
            unit: UnitRef {
                revision: rev(2),
                ..e.unit
            },
            ..e.clone()
        },
        ExpectedLifecycleBasis {
            attempt_id: Some(id(99)),
            ..e.clone()
        },
    ] {
        assert_eq!(
            plan_unit_attempt_transition(
                &state,
                &bad,
                scope(),
                row(1),
                &BTreeSet::new(),
                &UnitAttemptCommand::RequestCancellation {}
            ),
            Err(WorkError::ResultBasisMismatch)
        );
    }
    assert_eq!(
        plan_unit_attempt_transition(
            &state,
            &e,
            scope(),
            row(2),
            &BTreeSet::new(),
            &UnitAttemptCommand::RequestCancellation {}
        ),
        Err(WorkError::ResultBasisMismatch)
    );
    let late = plan_unit_attempt_transition(
        &state,
        &e,
        scope(),
        row(2),
        &BTreeSet::new(),
        &UnitAttemptCommand::ObserveSubmission { result_id: id(11) },
    )
    .unwrap();
    assert!(late.observation_only());
    assert_eq!(state.unit.stage, UnitStage::Running);
}

#[test]
fn cancellation_and_loss_preserve_late_observation_without_promoting_result() {
    for command in [
        UnitAttemptCommand::RequestCancellation {},
        UnitAttemptCommand::ObserveLost {
            observation_id: id(12),
        },
    ] {
        let running = started();
        let changed = step(&running, command)
            .unwrap()
            .replacement()
            .unwrap()
            .clone();
        let late = step(
            &changed,
            UnitAttemptCommand::ObserveSubmission { result_id: id(11) },
        )
        .unwrap();
        assert!(late.observation_only());
        assert!(matches!(
            late.events()[0],
            LifecycleEvent::LateObservation { .. }
        ));
        assert!(changed.unit.accepted_result.is_none());
    }
    let running = started();
    let cancelling = step(&running, UnitAttemptCommand::RequestCancellation {})
        .unwrap()
        .replacement()
        .unwrap()
        .clone();
    let cancelled = step(&cancelling, UnitAttemptCommand::ConfirmCancellation {})
        .unwrap()
        .replacement()
        .unwrap()
        .clone();
    assert_eq!(cancelled.unit.stage, UnitStage::Cancelled);
    assert!(step(&cancelled, UnitAttemptCommand::Start {}).is_err());
    assert!(
        step(
            &cancelled,
            UnitAttemptCommand::ObserveSubmission { result_id: id(11) }
        )
        .unwrap()
        .observation_only()
    );
}

#[test]
fn prepared_stop_blocks_start_but_does_not_block_cancellation_or_observation() {
    let running = started();
    let gates = BTreeSet::from([
        BlockingReason::StopRequested {},
        BlockingReason::Recovery {},
    ]);
    let late = plan_unit_attempt_transition(
        &running,
        &expected(&running),
        scope(),
        row(1),
        &gates,
        &UnitAttemptCommand::ObserveSubmission { result_id: id(11) },
    )
    .unwrap();
    assert!(late.observation_only());
    assert!(
        plan_unit_attempt_transition(
            &running,
            &expected(&running),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::RequestCancellation {}
        )
        .is_ok()
    );
    let ready = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    assert!(matches!(
        plan_unit_attempt_transition(
            &ready,
            &expected(&ready),
            scope(),
            row(1),
            &gates,
            &UnitAttemptCommand::Prepare {
                new_attempt_id: id(10)
            }
        ),
        Err(WorkError::Blocked(_))
    ));
    assert_eq!(gates.len(), 2);
}

#[test]
fn no_attempt_ready_cancellation_has_a_closed_completion_path() {
    let ready = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    let cancelling = step(&ready, UnitAttemptCommand::RequestCancellation {})
        .unwrap()
        .replacement()
        .unwrap()
        .clone();
    let cancelled = step(&cancelling, UnitAttemptCommand::ConfirmCancellation {})
        .unwrap()
        .replacement()
        .unwrap()
        .clone();
    assert_eq!(cancelled.unit.stage, UnitStage::Cancelled);
    assert!(cancelled.attempt.is_none());
}

#[test]
fn lifecycle_version_overflow_and_unknown_wire_fields_fail_closed() {
    let mut state = started();
    state.unit.row_version = row(u64::MAX);
    assert_eq!(
        step(&state, UnitAttemptCommand::RequestCancellation {}),
        Err(WorkError::VersionOverflow)
    );
    assert_eq!(state.unit.stage, UnitStage::Running);
    assert!(
        serde_json::from_str::<UnitAttemptCommand>(r#"{"kind":"start","authority":true}"#).is_err()
    );
    assert!(serde_json::from_str::<UnitAttemptCommand>(r#"{"kind":"accept_any_result"}"#).is_err());
}

#[test]
fn every_lifecycle_command_rejects_unknown_and_duplicate_wire_fields() {
    for command in [
        UnitAttemptCommand::Prepare {
            new_attempt_id: id(10),
        },
        UnitAttemptCommand::Start {},
        UnitAttemptCommand::ObserveSubmission { result_id: id(11) },
        UnitAttemptCommand::RequestCancellation {},
        UnitAttemptCommand::ConfirmCancellation {},
        UnitAttemptCommand::ObserveLost {
            observation_id: id(12),
        },
    ] {
        let encoded = serde_json::to_string(&command).unwrap();
        assert_eq!(
            serde_json::from_str::<UnitAttemptCommand>(&encoded).unwrap(),
            command
        );
        let prefix = &encoded[..encoded.len() - 1];
        let unknown = format!("{prefix},\"authority\":true}}");
        assert!(
            serde_json::from_str::<UnitAttemptCommand>(&unknown).is_err(),
            "accepted unknown field: {unknown}"
        );
        let duplicate_tag = format!("{prefix},\"kind\":\"start\"}}");
        assert!(
            serde_json::from_str::<UnitAttemptCommand>(&duplicate_tag).is_err(),
            "accepted duplicate tag: {duplicate_tag}"
        );
        let duplicate_field = match command {
            UnitAttemptCommand::Prepare { new_attempt_id } => Some(format!(
                "{prefix},\"new_attempt_id\":\"{new_attempt_id}\"}}"
            )),
            UnitAttemptCommand::ObserveSubmission { result_id } => {
                Some(format!("{prefix},\"result_id\":\"{result_id}\"}}"))
            }
            UnitAttemptCommand::ObserveLost { observation_id } => Some(format!(
                "{prefix},\"observation_id\":\"{observation_id}\"}}"
            )),
            _ => None,
        };
        if let Some(duplicate) = duplicate_field {
            assert!(serde_json::from_str::<UnitAttemptCommand>(&duplicate).is_err());
        }
    }
}

#[test]
fn impossible_composite_state_and_future_generation_are_rejected() {
    let mut state = started();
    state.attempt.as_mut().unwrap().stage = AttemptStage::Prepared;
    assert_eq!(
        step(&state, UnitAttemptCommand::RequestCancellation {}),
        Err(WorkError::InvalidField("unit_attempt_state"))
    );
    let mut ready = UnitAttemptState {
        unit: unit(),
        attempt: None,
    };
    ready.unit.accepted_result = Some(AcceptedResultRef {
        result_id: id(50),
        snapshot_id: id(51),
        fingerprint: [0; 32],
        result_row_version: row(1),
    });
    assert_eq!(
        step(
            &ready,
            UnitAttemptCommand::Prepare {
                new_attempt_id: id(10)
            }
        ),
        Err(WorkError::InvalidField("unit_attempt_state"))
    );
    let mut future = started();
    future.attempt.as_mut().unwrap().run_generation = row(3);
    let mut expected = expected(&future);
    expected.run_generation = row(3);
    assert_eq!(
        plan_unit_attempt_transition(
            &future,
            &expected,
            scope(),
            row(2),
            &BTreeSet::new(),
            &UnitAttemptCommand::ObserveSubmission { result_id: id(11) }
        ),
        Err(WorkError::ResultBasisMismatch)
    );
}

fn completion_fixture() -> (
    GraphCandidate,
    ResultPacket,
    VerificationRecord,
    GraphLimits,
) {
    let policy = VersionedDependency {
        key: DependencyKey {
            kind: DependencyKind::Policy,
            entity_id: id(20),
        },
        expected_revision: row(1),
        fingerprint: None,
    };
    let read = VersionedDependency {
        key: DependencyKey {
            kind: DependencyKind::Obligation,
            entity_id: id(5),
        },
        expected_revision: row(1),
        fingerprint: None,
    };
    let packet = ResultPacket {
        result_id: id(21),
        attempt_id: id(10),
        scope: scope(),
        unit: unit().unit,
        snapshot: DependencySnapshot {
            snapshot_id: id(22),
            scope: scope(),
            unit: unit().unit,
            read_set: vec![read],
            policy_versions: vec![policy.clone()],
        },
        fingerprint: [1; 32],
        verification_policy: policy.clone(),
        used_links: vec![],
        stage: ResultStage::Accepted,
        row_version: row(3),
    };
    let proof = VerificationRecord {
        verification_id: id(23),
        result_id: packet.result_id,
        result_fingerprint: packet.fingerprint,
        unit: packet.unit,
        snapshot_id: packet.snapshot.snapshot_id,
        verification_policy: policy.clone(),
        verifier_attempt_id: id(24),
        verdict: VerificationVerdict::Pass,
        findings: vec![ObligationFinding {
            obligation_id: id(5),
            status: ObligationStatus::Satisfied,
            evidence_id: id(25),
        }],
        evidence: VerificationEvidence {
            verification_plan_id: id(50),
            method_revision_id: id(53),
            required_level: VerificationLevel::IndependentReview,
            scope: scope(),
            work_kind: WorkKind::Research,
            criterion_ids: vec![id(5)],
            level: VerificationLevel::IndependentReview,
            machine_findings_id: id(51),
            policy: policy.clone(),
            qualification: Some(MethodQualification {
                qualification_id: id(52),
                method_revision_id: id(53),
                work_kind: WorkKind::Research,
                criterion_ids: vec![id(5)],
                level: VerificationLevel::IndependentReview,
                policy,
                status: QualificationStatus::Qualified,
            }),
            independence_assessment_id: Some(id(54)),
            protected_admission_evidence_id: None,
        },
    };
    let objective = ObjectiveContract {
        scope: scope(),
        statement: "Produce report".into(),
        owner_event_ids: vec![id(26)],
        obligations: vec![Obligation {
            obligation_id: id(5),
            objective_id: id(3),
            parent_id: None,
            kind: ObligationKind::Required,
            statement: "Quality criterion".into(),
            verification_method: VerificationMethod::Deterministic,
            status: ObligationStatus::Satisfied,
            waiver_owner_event_id: None,
            row_version: row(1),
        }],
        completion_criteria: vec![id(5)],
        row_version: row(1),
    };
    let mut unit = unit();
    unit.stage = UnitStage::Accepted;
    unit.accepted_result = Some(AcceptedResultRef {
        result_id: packet.result_id,
        snapshot_id: packet.snapshot.snapshot_id,
        fingerprint: packet.fingerprint,
        result_row_version: packet.row_version,
    });
    let graph = GraphCandidate {
        expected_graph_version: row(1),
        current_graph_version: row(1),
        roots: vec![root()],
        objectives: vec![objective],
        units: vec![unit],
        dependencies: vec![],
        links: vec![],
        publications: vec![],
        publication_states: vec![],
        publication_bindings: vec![],
        owner_acceptances: vec![],
    };
    let limits = GraphLimits {
        max_units: rev(10),
        max_edges: rev(10),
        max_attempts_per_unit: rev(3),
        max_steps_per_unit: rev(100),
        max_exploration_expansions: rev(10),
    };
    (graph, packet, proof, limits)
}

#[test]
fn root_lifecycle_requires_opaque_exact_fresh_candidate_review_and_owner_decision() {
    let (g, packet, proof, limits) = completion_fixture();
    let graph = validate_work_graph(&g, &limits).unwrap();
    let mut rows = graph_dependency_rows(&graph);
    rows.push(packet.verification_policy.clone());
    let evidence = [AcceptedResultEvidence {
        packet: &packet,
        verification: &proof,
    }];
    let view = CompletionCheckView {
        graph: &graph,
        results: &evidence,
        current_rows: &rows,
    };
    let candidate = derive_completion_candidate(id(30), id(3), &graph, &evidence, &rows).unwrap();
    let objective = validate_objective(&g.objectives[0], &OwnerConstraints::default()).unwrap();
    let mut state = RootLifecycleState {
        root: root(),
        scope: scope(),
        candidate: None,
        final_review: None,
        pending_owner_rejection: None,
    };
    state = plan_root_transition(
        &state,
        state.root.row_version,
        scope(),
        None,
        &BTreeSet::new(),
        RootLifecycleCommand::Start(&objective),
    )
    .unwrap()
    .replacement()
    .clone();
    assert!(
        plan_root_transition(
            &state,
            state.root.row_version,
            scope(),
            None,
            &BTreeSet::new(),
            RootLifecycleCommand::PresentCandidate(&candidate)
        )
        .is_err()
    );
    state = plan_root_transition(
        &state,
        state.root.row_version,
        scope(),
        Some(&view),
        &BTreeSet::new(),
        RootLifecycleCommand::PresentCandidate(&candidate),
    )
    .unwrap()
    .replacement()
    .clone();
    state = plan_root_transition(
        &state,
        state.root.row_version,
        scope(),
        Some(&view),
        &BTreeSet::new(),
        RootLifecycleCommand::BeginFinalReview,
    )
    .unwrap()
    .replacement()
    .clone();
    let final_evidence = FinalReviewEvidence {
        verification_id: id(31),
        candidate_id: id(30),
        scope: scope(),
        reviewed_results: candidate.results().to_vec(),
        verdict: VerificationVerdict::Pass,
        source_verification: Some(test_verification_record(id(31), &unit())),
    };
    let reviewed = validate_final_review(&candidate, &final_evidence, &unit(), &test_objective()).unwrap();
    state = plan_root_transition(
        &state,
        state.root.row_version,
        scope(),
        Some(&view),
        &BTreeSet::new(),
        RootLifecycleCommand::CompleteFinalReview(&reviewed),
    )
    .unwrap()
    .replacement()
    .clone();
    assert_eq!(state.root.stage, RootStage::AwaitingOwnerAcceptance);
    state
        .root
        .blocking_reasons
        .insert(BlockingReason::AwaitingOwner {});
    let decision = OwnerAcceptanceDecision {
        owner_event_id: id(32),
        candidate_id: id(30),
        scope: scope(),
        reviewed_results: candidate.results().to_vec(),
        final_verification_id: id(31),
        final_verdict: VerificationVerdict::Pass,
        accepted_owner_obligations: vec![],
    };
    let accepted = validate_owner_acceptance(&candidate, &decision).unwrap();
    let other_candidate =
        derive_completion_candidate(id(40), id(3), &graph, &evidence, &rows).unwrap();
    let other_review = validate_final_review(
        &other_candidate,
        &FinalReviewEvidence {
            candidate_id: id(40),
            source_verification: Some(test_verification_record(id(41), &unit())),
            ..final_evidence.clone()
        },
        &unit(),
        &test_objective(),
    )
    .unwrap();
    let other_accepted = validate_owner_acceptance(
        &other_candidate,
        &OwnerAcceptanceDecision {
            candidate_id: id(40),
            ..decision
        },
    )
    .unwrap();
    let mut mixed = state.clone();
    mixed.final_review = Some(other_review);
    assert_eq!(
        plan_root_transition(
            &mixed,
            mixed.root.row_version,
            scope(),
            Some(&view),
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&other_accepted)
        )
        .unwrap_err(),
        WorkError::ResultBasisMismatch
    );
    let mut foreign_scope = state.clone();
    foreign_scope.scope.objective_revision = rev(2);
    assert_eq!(
        plan_root_transition(
            &foreign_scope,
            foreign_scope.root.row_version,
            foreign_scope.scope,
            Some(&view),
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        )
        .unwrap_err(),
        WorkError::ResultBasisMismatch
    );
    let mut stale_rows = rows.clone();
    stale_rows
        .iter_mut()
        .find(|r| r.key.kind == DependencyKind::Obligation)
        .unwrap()
        .expected_revision = row(2);
    let stale_view = CompletionCheckView {
        current_rows: &stale_rows,
        ..view
    };
    assert_eq!(
        plan_root_transition(
            &state,
            state.root.row_version,
            scope(),
            Some(&stale_view),
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        )
        .unwrap_err(),
        WorkError::ResultBasisMismatch
    );
    let mut stopped = state.clone();
    stopped
        .root
        .blocking_reasons
        .insert(BlockingReason::StopRequested {});
    assert!(matches!(
        plan_root_transition(
            &stopped,
            stopped.root.row_version,
            scope(),
            Some(&view),
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        ),
        Err(WorkError::Blocked(_))
    ));
    let done = plan_root_transition(
        &state,
        state.root.row_version,
        scope(),
        Some(&view),
        &BTreeSet::new(),
        RootLifecycleCommand::AcceptByOwner(&accepted),
    )
    .unwrap();
    assert_eq!(done.replacement().root.stage, RootStage::OwnerAccepted);
    assert_eq!(
        done.replacement().root.blocking_reasons,
        state.root.blocking_reasons,
        "this pure planner never silently clears a gate"
    );
    assert_eq!(
        done.event(),
        &LifecycleEvent::OwnerCompletionAccepted {
            work_id: id(2),
            candidate_id: id(30),
            owner_event_id: id(32),
            final_verification_id: id(31)
        }
    );
    assert_eq!(state.root.stage, RootStage::AwaitingOwnerAcceptance);
    assert!(
        plan_root_transition(
            &state,
            row(1),
            scope(),
            Some(&view),
            &BTreeSet::new(),
            RootLifecycleCommand::AcceptByOwner(&accepted)
        )
        .is_err()
    );
}

#[test]
fn final_owner_acceptance_obeys_every_combination_of_root_and_installation_gates() {
    let (g, packet, proof, limits) = completion_fixture();
    let graph = validate_work_graph(&g, &limits).unwrap();
    let mut current_rows = graph_dependency_rows(&graph);
    current_rows.push(packet.verification_policy.clone());
    let results = [AcceptedResultEvidence {
        packet: &packet,
        verification: &proof,
    }];
    let view = CompletionCheckView {
        graph: &graph,
        results: &results,
        current_rows: &current_rows,
    };
    let candidate =
        derive_completion_candidate(id(100), id(3), &graph, &results, &current_rows).unwrap();
    let review = validate_final_review(
        &candidate,
        &FinalReviewEvidence {
            verification_id: id(101),
            candidate_id: id(100),
            scope: scope(),
            reviewed_results: candidate.results().to_vec(),
            verdict: VerificationVerdict::Pass,
            source_verification: Some(test_verification_record(id(101), &unit())),
        },
        &unit(),
        &test_objective(),
    )
    .unwrap();
    let accepted = validate_owner_acceptance(
        &candidate,
        &OwnerAcceptanceDecision {
            owner_event_id: id(102),
            candidate_id: id(100),
            scope: scope(),
            reviewed_results: candidate.results().to_vec(),
            final_verification_id: id(101),
            final_verdict: VerificationVerdict::Pass,
            accepted_owner_obligations: vec![],
        },
    )
    .unwrap();
    let rows = gate_contract_rows();
    for mask in 0_u16..(1 << rows.len()) {
        let mut state = RootLifecycleState {
            root: root(),
            scope: scope(),
            candidate: Some(candidate.clone()),
            final_review: Some(review.clone()),
            pending_owner_rejection: None,
        };
        state.root.stage = RootStage::AwaitingOwnerAcceptance;
        let mut installation = BTreeSet::new();
        let mut expected_blockers = BTreeSet::new();
        for (index, (reason, flags)) in rows.iter().enumerate() {
            if mask & (1 << index) != 0 {
                if index % 2 == 0 {
                    installation.insert(reason.clone());
                } else {
                    state.root.blocking_reasons.insert(reason.clone());
                }
                if flags[10] == 0 {
                    expected_blockers.insert(reason.clone());
                }
            }
        }
        let plan = plan_root_transition(
            &state,
            state.root.row_version,
            scope(),
            Some(&view),
            &installation,
            RootLifecycleCommand::AcceptByOwner(&accepted),
        );
        if let Some(blocker) = expected_blockers.into_iter().next() {
            assert_eq!(
                plan.unwrap_err(),
                WorkError::Blocked(blocker),
                "mask={mask:#06x}"
            );
        } else {
            let plan = plan.unwrap();
            assert_eq!(plan.replacement().root.stage, RootStage::OwnerAccepted);
            assert_eq!(plan.replacement().root.row_version, row(2));
            assert_eq!(
                plan.replacement().root.blocking_reasons,
                state.root.blocking_reasons
            );
            assert_eq!(
                plan.event(),
                &LifecycleEvent::OwnerCompletionAccepted {
                    work_id: id(2),
                    candidate_id: id(100),
                    owner_event_id: id(102),
                    final_verification_id: id(101),
                }
            );
        }
        assert_eq!(state.root.stage, RootStage::AwaitingOwnerAcceptance);
    }
}
