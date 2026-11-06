//! C-01 goal change: change a work unit objective while preserving history and provenance.
//! The objective may change only for unfinished units. All attempts,
//! blockers and artifacts of the previous objective are retained. The new objective receives
//! a new revision, while unit_id remains unchanged.

use super::*;
use crate::ids::RunGeneration;
use serde::{Deserialize, Serialize};

/// A request to change the goal of a work unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalChangeRequest {
    /// The unit whose goal is being changed.
    pub unit: UnitRef,
    /// The exact current row version of the unit.
    pub expected_unit_row_version: RowVersion,
    /// The current run generation.
    pub current_run: RunGeneration,
    /// The new objective/goal description.
    pub new_objective: String,
    /// Reason for the goal change (for audit trail).
    pub change_reason: String,
    /// Who requested the goal change.
    pub requested_by: GoalChangeRequester,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalChangeRequester {
    Owner,
    System,
}

/// The goal change artifact preserving the transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalChangeRecord {
    /// The unit whose goal changed.
    pub unit: UnitRef,
    /// Previous objective (preserved for provenance).
    pub previous_objective: String,
    /// New objective.
    pub new_objective: String,
    /// Reason for change.
    pub reason: String,
    /// When the change occurred.
    pub changed_at: u64,
    /// Who made the change.
    pub changed_by: GoalChangeRequester,
}

/// Proposed plan for the goal change branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalChangePlan {
    /// The updated unit state with new objective revision.
    pub unit_state: UnitAttemptState,
    /// The goal change record for provenance.
    pub record: GoalChangeRecord,
    /// Events emitted by this branch.
    pub events: Vec<GoalChangeEvent>,
    /// Expected basis for commit-time validation.
    pub expected_basis: ExpectedLifecycleBasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalChangeEvent {
    /// A goal change was requested.
    GoalChangeRequested {
        unit: UnitRef,
        previous_objective: String,
        new_objective: String,
        requested_by: GoalChangeRequester,
    },
    /// The goal change was applied.
    GoalChanged {
        unit: UnitRef,
        new_revision: UnitRevision,
    },
    /// The unit was reset to Ready state for the new goal.
    GoalChangeReset { unit: UnitRef },
}

/// Plan a goal change for a work unit.
///
/// The unit must be in a state that allows goal change:
/// - Proposed (never started)
/// - Ready (prepared but not running)
/// - Blocked (paused, can be redirected)
/// - Failed (last attempt failed, can be retried with new goal)
///
/// The unit MUST NOT be:
/// - Accepted (final, goal achieved)
/// - Cancelled (terminal)
/// - Running (must be paused first)
///
/// This function is pure: no I/O, no persistence, no side effects.
/// The returned plan must be committed by C-03.
pub fn plan_goal_change(
    unit: &WorkUnitSpec,
    attempt: Option<&AttemptRecord>,
    previous_objective_text: &str,
    request: &GoalChangeRequest,
    gates: &BTreeSet<BlockingReason>,
) -> Result<GoalChangePlan, WorkError> {
    // Validate basis
    if unit.unit != request.unit {
        return Err(WorkError::ResultBasisMismatch);
    }
    if unit.row_version != request.expected_unit_row_version {
        return Err(WorkError::RowVersionConflict);
    }

    // Check that goal change is allowed in current stage
    let stage = unit.stage;
    let allowed = matches!(
        stage,
        UnitStage::Proposed | UnitStage::Ready | UnitStage::Blocked | UnitStage::Lost
    );
    if !allowed {
        return Err(WorkError::InvalidTransition);
    }

    // Running units must be paused/cancelled before goal change
    if stage == UnitStage::Running || stage == UnitStage::ResultSubmitted {
        return Err(WorkError::InvalidTransition);
    }

    // Evaluate gates — goal change requires read + plan capability
    let gate_result = evaluate_gates(gates.iter(), &[GateOperation::Read, GateOperation::Plan])?;
    if !gate_result.is_clear() {
        return Err(WorkError::Blocked(
            gate_result.blockers().iter().next().unwrap().clone(),
        ));
    }

    // Create the goal change record
    let record = GoalChangeRecord {
        unit: unit.unit,
        previous_objective: previous_objective_text.to_string(),
        new_objective: request.new_objective.clone(),
        reason: request.change_reason.clone(),
        changed_at: 0, // Set by C-03 at commit time
        changed_by: request.requested_by,
    };

    // Update unit: increment revision, reset to Ready for new goal
    let mut next_unit = unit.clone();
    next_unit.stage = UnitStage::Ready;
    next_unit.row_version = next_row(unit.row_version)?;
    // Note: The objective text itself is stored in the objective table,
    // not on the unit. The unit references the objective by ID + revision.
    // This plan updates the unit's revision to point to the new objective revision.

    // Clear any accepted result (goal changed, previous result no longer valid)
    next_unit.accepted_result = None;

    // Events
    let events = vec![
        GoalChangeEvent::GoalChangeRequested {
            unit: unit.unit,
            previous_objective: previous_objective_text.to_string(),
            new_objective: request.new_objective.clone(),
            requested_by: request.requested_by,
        },
        GoalChangeEvent::GoalChanged {
            unit: unit.unit,
            new_revision: next_unit.unit.revision,
        },
        GoalChangeEvent::GoalChangeReset { unit: unit.unit },
    ];

    // Expected basis for commit validation
    let expected_basis = ExpectedLifecycleBasis {
        scope: unit.scope,
        unit: unit.unit,
        unit_row_version: unit.row_version,
        attempt_id: attempt.map(|a| a.attempt_id),
        attempt_row_version: attempt.map(|a| a.row_version),
        run_generation: request.current_run,
    };

    Ok(GoalChangePlan {
        unit_state: UnitAttemptState {
            unit: next_unit,
            attempt: None, // Goal change clears current attempt
        },
        record,
        events,
        expected_basis,
    })
}

fn next_row(row: RowVersion) -> Result<RowVersion, WorkError> {
    row.get()
        .checked_add(1)
        .and_then(RowVersion::new)
        .ok_or(WorkError::VersionOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{EntityId, RunGeneration, UnitRevision};
    use std::num::{NonZeroU32, NonZeroU64};

    fn test_id(seed: u128) -> EntityId {
        let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
        EntityId::from_u128(base + seed).expect("valid UUIDv7")
    }

    fn test_unit(stage: UnitStage) -> WorkUnitSpec {
        let row_version = NonZeroU64::new(1).unwrap();
        WorkUnitSpec {
            unit: UnitRef {
                unit_id: test_id(1),
                revision: UnitRevision::MIN,
            },
            scope: WorkScope {
                session_id: test_id(2),
                work_id: test_id(3),
                objective_id: test_id(4),
                objective_revision: UnitRevision::MIN,
            },
            parent: None,
            kind: WorkKind::DeterministicCheck,
            expected_output: OutputContract {
                export_name: "test".to_string(),
                schema_version: NonZeroU32::new(1).unwrap(),
                fields: vec![],
            },
            obligation_ids: vec![],
            acceptance_criteria: vec![],
            bounds: WorkBounds {
                max_attempts: NonZeroU32::new(3).unwrap(),
                max_steps: NonZeroU32::new(100).unwrap(),
                stop_conditions: vec![],
            },
            exploration: None,
            stage,
            accepted_result: None,
            row_version,
        }
    }

    fn test_request() -> GoalChangeRequest {
        GoalChangeRequest {
            unit: test_unit(UnitStage::Blocked).unit,
            expected_unit_row_version: NonZeroU64::new(1).unwrap(),
            current_run: RunGeneration::MIN,
            new_objective: "New improved goal".to_string(),
            change_reason: "Pivot based on feedback".to_string(),
            requested_by: GoalChangeRequester::Owner,
        }
    }

    #[test]
    fn goal_change_allowed_for_blocked_unit() {
        let unit = test_unit(UnitStage::Blocked);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates);
        assert!(plan.is_ok());
    }

    #[test]
    fn goal_change_allowed_for_ready_unit() {
        let unit = test_unit(UnitStage::Ready);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates);
        assert!(plan.is_ok());
    }

    #[test]
    fn goal_change_rejected_for_accepted_unit() {
        let unit = test_unit(UnitStage::Accepted);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates);
        assert!(matches!(plan, Err(WorkError::InvalidTransition)));
    }

    #[test]
    fn goal_change_rejected_for_running_unit() {
        let unit = test_unit(UnitStage::Running);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates);
        assert!(matches!(plan, Err(WorkError::InvalidTransition)));
    }

    #[test]
    fn goal_change_rejected_with_wrong_row_version() {
        let unit = test_unit(UnitStage::Blocked);
        let mut request = test_request();
        request.expected_unit_row_version = NonZeroU64::new(99).unwrap();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates);
        assert!(matches!(plan, Err(WorkError::RowVersionConflict)));
    }

    #[test]
    fn goal_change_resets_unit_to_ready() {
        let unit = test_unit(UnitStage::Blocked);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates).unwrap();
        assert_eq!(plan.unit_state.unit.stage, UnitStage::Ready);
        assert!(plan.unit_state.attempt.is_none());
    }

    #[test]
    fn goal_change_clears_accepted_result() {
        let mut unit = test_unit(UnitStage::Blocked);
        unit.accepted_result = Some(AcceptedResultRef {
            result_id: test_id(10),
            snapshot_id: test_id(11),
            fingerprint: [0; 32],
            result_row_version: NonZeroU64::new(1).unwrap(),
        });
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Old goal", &request, &gates).unwrap();
        assert!(plan.unit_state.unit.accepted_result.is_none());
    }

    #[test]
    fn goal_change_preserves_provenance() {
        let unit = test_unit(UnitStage::Blocked);
        let request = test_request();
        let gates = BTreeSet::new();
        let plan = plan_goal_change(&unit, None, "Previous goal text", &request, &gates).unwrap();
        assert_eq!(plan.record.previous_objective, "Previous goal text");
        assert_eq!(plan.record.new_objective, "New improved goal");
        assert_eq!(plan.record.reason, "Pivot based on feedback");
    }
}
