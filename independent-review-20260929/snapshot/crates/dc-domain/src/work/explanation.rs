//! C-01 explanation branch: a work unit can be explained/clarified without
//! being repaired or rejected. This is a non-destructive pause for semantic
//! clarification. The unit preserves all state, blockers, and provenance.
//! No progress is made, no resources are spent, no side effects occur.

use super::*;
use crate::ids::RunGeneration;
use serde::{Deserialize, Serialize};

/// An explanation request for a blocked or running work unit.
/// The unit is not modified; only a clarification artifact is proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationRequest {
    /// The unit to explain.
    pub unit: UnitRef,
    /// The exact current row version of the unit.
    pub expected_unit_row_version: RowVersion,
    /// The current run generation.
    pub current_run: RunGeneration,
    /// What needs explanation (free-form, bounded by MAX_TEXT_BYTES).
    pub explanation_topic: String,
    /// Who requested the explanation (owner, verifier, or system).
    pub requested_by: ExplanationRequester,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExplanationRequester {
    Owner,
    Verifier,
    System,
}

/// The explanation artifact produced by the branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplanationArtifact {
    /// The unit that was explained.
    pub unit: UnitRef,
    /// The explanation content.
    pub content: String,
    /// The revision of the unit at explanation time.
    pub unit_revision: UnitRevision,
    /// When the explanation was produced.
    pub explained_at: u64,
}

/// Proposed plan for the explanation branch.
/// Contains no state transition — the unit remains in its current stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationPlan {
    /// The unchanged unit state (proof that no destructive transition occurs).
    pub unit_state: UnitAttemptState,
    /// The proposed explanation artifact.
    pub artifact: ExplanationArtifact,
    /// Events emitted by this branch.
    pub events: Vec<ExplanationEvent>,
    /// Expected basis for commit-time validation.
    pub expected_basis: ExpectedLifecycleBasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplanationEvent {
    /// An explanation was requested.
    ExplanationRequested {
        unit: UnitRef,
        topic: String,
        requested_by: ExplanationRequester,
    },
    /// An explanation was produced.
    ExplanationProduced {
        unit: UnitRef,
        artifact_id: EntityId,
    },
    /// The unit resumed after explanation (no state change, just a marker).
    ExplanationAcknowledged { unit: UnitRef },
}

/// Plan an explanation branch for a work unit.
///
/// The unit must be in a state that allows explanation:
/// - Blocked (any blocking reason)
/// - Running (for progress clarification)
/// - ResultSubmitted (for result clarification)
///
/// The unit MUST NOT be:
/// - Accepted (final, no explanation needed)
/// - Cancelled (terminal)
/// - Lost (terminal)
///
/// This function is pure: no I/O, no persistence, no side effects.
/// The returned plan must be committed by C-03.
pub fn plan_explanation(
    unit: &WorkUnitSpec,
    attempt: Option<&AttemptRecord>,
    request: &ExplanationRequest,
    gates: &BTreeSet<BlockingReason>,
) -> Result<ExplanationPlan, WorkError> {
    // Validate basis
    if unit.unit != request.unit {
        return Err(WorkError::ResultBasisMismatch);
    }
    if unit.row_version != request.expected_unit_row_version {
        return Err(WorkError::RowVersionConflict);
    }

    // Check that explanation is allowed in current stage
    let stage = unit.stage;
    let allowed = matches!(
        stage,
        UnitStage::Blocked | UnitStage::Running | UnitStage::ResultSubmitted | UnitStage::Verifying
    );
    if !allowed {
        return Err(WorkError::InvalidTransition);
    }

    // Evaluate gates — explanation is a read/reason operation, not an effect
    let gate_result = evaluate_gates(gates.iter(), &[GateOperation::Read, GateOperation::Reason])?;
    if !gate_result.is_clear() {
        return Err(WorkError::Blocked(
            gate_result.blockers().iter().next().unwrap().clone(),
        ));
    }

    // Build the artifact
    let artifact = ExplanationArtifact {
        unit: unit.unit,
        content: format!("Explanation requested for: {}", request.explanation_topic),
        unit_revision: unit.unit.revision,
        explained_at: 0,
    };

    // Events
    let events = vec![
        ExplanationEvent::ExplanationRequested {
            unit: unit.unit,
            topic: request.explanation_topic.clone(),
            requested_by: request.requested_by,
        },
        ExplanationEvent::ExplanationProduced {
            unit: unit.unit,
            artifact_id: unit.unit.unit_id, // Placeholder — real ID from C-03
        },
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

    Ok(ExplanationPlan {
        unit_state: UnitAttemptState {
            unit: unit.clone(),
            attempt: attempt.cloned(),
        },
        artifact,
        events,
        expected_basis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{EntityId, RunGeneration, UnitRevision};
    use std::num::{NonZeroU32, NonZeroU64};

    fn test_id(seed: u128) -> EntityId {
        // UUIDv7: version=7 (bits 76-79 = 0111), variant=2 (bits 62-63 = 10)
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

    fn test_requester() -> ExplanationRequester {
        ExplanationRequester::Owner
    }

    #[test]
    fn explanation_allowed_for_blocked_unit() {
        let unit = test_unit(UnitStage::Blocked);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "Why is this blocked?".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(plan.is_ok());
    }

    #[test]
    fn explanation_allowed_for_running_unit() {
        let unit = test_unit(UnitStage::Running);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "Progress clarification".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(plan.is_ok());
    }

    #[test]
    fn explanation_rejected_for_accepted_unit() {
        let unit = test_unit(UnitStage::Accepted);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "Too late".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(matches!(plan, Err(WorkError::InvalidTransition)));
    }

    #[test]
    fn explanation_rejected_for_cancelled_unit() {
        let unit = test_unit(UnitStage::Cancelled);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "Too late".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(matches!(plan, Err(WorkError::InvalidTransition)));
    }

    #[test]
    fn explanation_rejected_with_wrong_row_version() {
        let unit = test_unit(UnitStage::Blocked);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: NonZeroU64::new(2).unwrap(),
            current_run: RunGeneration::MIN,
            explanation_topic: "Stale read".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(matches!(plan, Err(WorkError::RowVersionConflict)));
    }

    #[test]
    fn explanation_rejected_with_blocking_gate() {
        let unit = test_unit(UnitStage::Blocked);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "Blocked by gate".to_string(),
            requested_by: test_requester(),
        };
        let mut gates = BTreeSet::new();
        gates.insert(BlockingReason::StopRequested {});
        let plan = plan_explanation(&unit, None, &request, &gates);
        assert!(matches!(plan, Err(WorkError::Blocked(_))));
    }

    #[test]
    fn explanation_preserves_unit_state() {
        let unit = test_unit(UnitStage::Blocked);
        let request = ExplanationRequest {
            unit: unit.unit,
            expected_unit_row_version: unit.row_version,
            current_run: RunGeneration::MIN,
            explanation_topic: "State check".to_string(),
            requested_by: test_requester(),
        };
        let gates = BTreeSet::new();
        let plan = plan_explanation(&unit, None, &request, &gates).unwrap();
        assert_eq!(plan.unit_state.unit.stage, UnitStage::Blocked);
        assert_eq!(plan.unit_state.unit.row_version, unit.row_version);
    }
}
