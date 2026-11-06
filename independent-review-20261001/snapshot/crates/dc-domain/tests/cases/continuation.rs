use super::*;

#[test]
fn continued_attempt_can_start_and_still_obeys_late_stop() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    let before = (unit.clone(), previous.clone());
    let plan =
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1)).unwrap();
    let state = UnitAttemptState {
        unit: plan.unit().clone(),
        attempt: Some(plan.attempt().clone()),
    };
    let basis = expected(&state);
    let stopped = BTreeSet::from([BlockingReason::StopRequested {}]);
    assert!(matches!(
        plan_unit_attempt_transition(
            &state,
            &basis,
            scope(),
            row(1),
            &stopped,
            &UnitAttemptCommand::Start {}
        ),
        Err(WorkError::Blocked(_))
    ));
    // Old row basis cannot authorize this new prepared attempt.
    assert!(
        plan_unit_attempt_transition(
            &state,
            plan.expected_basis(),
            scope(),
            row(1),
            &BTreeSet::new(),
            &UnitAttemptCommand::Start {}
        )
        .is_err()
    );
    let started = plan_unit_attempt_transition(
        &state,
        &basis,
        scope(),
        row(1),
        &BTreeSet::new(),
        &UnitAttemptCommand::Start {},
    )
    .unwrap();
    let running = started.replacement().unwrap();
    assert_eq!(running.unit.stage, UnitStage::Running);
    assert_eq!(
        running.attempt.as_ref().unwrap().stage,
        AttemptStage::Running
    );
    assert_eq!(running.attempt.as_ref().unwrap().attempt_id, id(11));
    assert_eq!((unit, previous), before);
}

#[test]
fn only_a_coherent_confirmed_failure_enters_this_partial_retry_path() {
    let unit_stages = [
        UnitStage::Proposed,
        UnitStage::Ready,
        UnitStage::Leased,
        UnitStage::Running,
        UnitStage::ResultSubmitted,
        UnitStage::Verifying,
        UnitStage::Accepted,
        UnitStage::Rejected,
        UnitStage::Stale,
        UnitStage::Cancelling,
        UnitStage::Cancelled,
        UnitStage::Lost,
        UnitStage::Blocked,
    ];
    let attempt_stages = [
        AttemptStage::Prepared,
        AttemptStage::Running,
        AttemptStage::Submitted,
        AttemptStage::Failed,
        AttemptStage::Lost,
        AttemptStage::Cancelled,
    ];
    for stage in unit_stages {
        for attempt_stage in attempt_stages {
            let unit = WorkUnitSpec { stage, ..unit() };
            let previous = ended_attempt(attempt_stage);
            let result =
                plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1));
            assert_eq!(
                result.is_ok(),
                stage == UnitStage::Blocked && attempt_stage == AttemptStage::Failed,
                "{stage:?}/{attempt_stage:?}"
            );
        }
    }
}

#[test]
fn invalid_count_result_and_version_cannot_be_silently_repaired() {
    let mut unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 0)),
        Err(WorkError::InvalidField("attempts_used"))
    );
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], u32::MAX)),
        Err(WorkError::VersionOverflow)
    );
    let accepted = AcceptedResultRef {
        result_id: id(90),
        snapshot_id: id(91),
        fingerprint: [0; 32],
        result_row_version: row(1),
    };
    unit.accepted_result = Some(accepted.clone());
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1)),
        Err(WorkError::InvalidField("unit_attempt_state"))
    );
    assert_eq!(unit.accepted_result, Some(accepted));
    unit.accepted_result = None;
    unit.row_version = row(u64::MAX);
    let mut context = ctx(&[], &[], &[], 1);
    context.expected_unit_row_version = unit.row_version;
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &context),
        Err(WorkError::VersionOverflow)
    );
}

#[test]
fn unknown_outcome_at_any_scope_prevents_preparing_retry() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    let gates = [BlockingReason::UnknownOutcome {}];
    for context in [
        ctx(&gates, &[], &[], 1),
        ctx(&[], &gates, &[], 1),
        ctx(&[], &[], &gates, 1),
    ] {
        assert_eq!(
            plan_continuation_new_attempt(&unit, &previous, id(11), &context),
            Err(WorkError::Blocked(BlockingReason::UnknownOutcome {}))
        );
    }
}

fn ended_attempt(stage: AttemptStage) -> AttemptRecord {
    AttemptRecord {
        attempt_id: id(10),
        scope: scope(),
        unit: unit().unit,
        run_generation: row(1),
        stage,
        row_version: row(1),
    }
}
fn ctx<'a>(
    installation_gates: &'a [BlockingReason],
    root_gates: &'a [BlockingReason],
    unit_gates: &'a [BlockingReason],
    attempts_used: u32,
) -> ContinuationContext<'a> {
    ContinuationContext {
        current_run: row(1),
        installation_gates,
        root_gates,
        unit_gates,
        expected_unit_row_version: row(1),
        expected_attempt_row_version: row(1),
        attempts_used,
    }
}

#[test]
fn retry_hands_continuation_to_a_new_prepared_attempt() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    let plan =
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1)).unwrap();
    // New attempt is a distinct identity, prepared, never the old attempt revived.
    assert_eq!(plan.attempt().attempt_id, id(11));
    assert_eq!(plan.attempt().stage, AttemptStage::Prepared);
    assert_eq!(plan.attempt().run_generation, row(1));
    assert_eq!(plan.attempt().row_version, RowVersion::MIN);
    // A prepared attempt belongs to a leased unit, exactly as in Prepare.
    assert_eq!(plan.unit().stage, UnitStage::Leased);
    assert_eq!(plan.unit().row_version, row(2));
    assert!(plan.unit().accepted_result.is_none());
    // Basis pins the exact previous unit and attempt rows.
    assert_eq!(plan.expected_basis().unit_row_version, row(1));
    assert_eq!(plan.expected_basis().attempt_id, Some(id(10)));
    assert_eq!(plan.expected_attempts_used(), 1);
    assert_eq!(plan.attempt_number(), 2);
}

#[test]
fn the_old_attempt_is_never_revived_or_reused() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    for stage in [
        AttemptStage::Prepared,
        AttemptStage::Running,
        AttemptStage::Submitted,
    ] {
        let previous = ended_attempt(stage);
        assert_eq!(
            plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1),),
            Err(WorkError::InvalidTransition),
            "non-terminal {stage:?} attempt must not be handed over"
        );
    }
    // Same attempt id cannot be re-prepared as the "new" attempt.
    let previous = ended_attempt(AttemptStage::Failed);
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(10), &ctx(&[], &[], &[], 1),),
        Err(WorkError::Duplicate(id(10)))
    );
}

#[test]
fn stale_unit_or_attempt_basis_is_rejected() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    let mut stale_unit = ctx(&[], &[], &[], 1);
    stale_unit.expected_unit_row_version = row(2);
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &stale_unit),
        Err(WorkError::RowVersionConflict)
    );
    let mut stale_attempt = ctx(&[], &[], &[], 1);
    stale_attempt.expected_attempt_row_version = row(9);
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &stale_attempt),
        Err(WorkError::RowVersionConflict)
    );
}

#[test]
fn foreign_scope_generation_or_unit_are_rejected() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    // Attempt belonging to a different unit.
    let foreign_unit_attempt = AttemptRecord {
        unit: UnitRef {
            unit_id: id(99),
            revision: rev(1),
        },
        ..previous.clone()
    };
    assert_eq!(
        plan_continuation_new_attempt(&unit, &foreign_unit_attempt, id(11), &ctx(&[], &[], &[], 1)),
        Err(WorkError::ResultBasisMismatch)
    );
    // Attempt from a different run generation.
    let foreign_run_attempt = AttemptRecord {
        run_generation: row(2),
        ..previous.clone()
    };
    assert_eq!(
        plan_continuation_new_attempt(&unit, &foreign_run_attempt, id(11), &ctx(&[], &[], &[], 1)),
        Err(WorkError::ResultBasisMismatch)
    );
    // Attempt scope bound to a different work id.
    let mut foreign_scope = scope();
    foreign_scope.work_id = id(77);
    let foreign_scope_attempt = AttemptRecord {
        scope: foreign_scope,
        ..previous.clone()
    };
    assert_eq!(
        plan_continuation_new_attempt(
            &unit,
            &foreign_scope_attempt,
            id(11),
            &ctx(&[], &[], &[], 1)
        ),
        Err(WorkError::ResultBasisMismatch)
    );
}

#[test]
fn attempt_limit_is_a_hard_budget_that_restart_does_not_reset() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    }; // max_attempts = 2
    let previous = ended_attempt(AttemptStage::Failed);
    // Retry with the budget already exhausted is refused, not silently
    // allowed, and a restart does not reset the spent budget (doc 16 §9.2).
    assert_eq!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 2)),
        Err(WorkError::Blocked(BlockingReason::ResourceUnavailable {}))
    );
    // One attempt remains: the next (second) attempt fits the budget.
    assert!(
        plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1)).is_ok()
    );
}

#[test]
fn protective_gates_block_the_continuation_handoff() {
    let unit = WorkUnitSpec {
        stage: UnitStage::Blocked,
        ..unit()
    };
    let previous = ended_attempt(AttemptStage::Failed);
    for gate in [
        BlockingReason::StopRequested {},
        BlockingReason::Recovery {},
        BlockingReason::IntegrityFailure {},
        BlockingReason::UnknownOutcome {},
    ] {
        assert!(
            matches!(
                plan_continuation_new_attempt(
                    &unit,
                    &previous,
                    id(11),
                    &ctx(&[], std::slice::from_ref(&gate), &[], 1),
                ),
                Err(WorkError::Blocked(_))
            ),
            "{gate:?} must block the handoff"
        );
    }
}

#[test]
fn a_unit_with_a_submitted_or_terminal_result_is_not_continued() {
    let previous = ended_attempt(AttemptStage::Failed);
    for stage in [
        UnitStage::Running,
        UnitStage::ResultSubmitted,
        UnitStage::Accepted,
        UnitStage::Cancelled,
    ] {
        let unit = WorkUnitSpec { stage, ..unit() };
        assert_eq!(
            plan_continuation_new_attempt(&unit, &previous, id(11), &ctx(&[], &[], &[], 1)),
            Err(WorkError::InvalidTransition),
            "{stage:?} unit must not be continued"
        );
    }
}
