//! Planner — derive a `GraphCandidate` from an `ObjectiveContract`.
//!
//! This is the bridge that was missing between "owner submits an objective"
//! and "owner reviews a work graph". Today it produces the minimal valid
//! graph: one root work unit derived directly from the objective statement.
//!
//! Design rules:
//! - Pure function: no I/O, no clock, no randomness, no global state.
//! - Deterministic IDs derived from the objective scope so repeated planning
//!   of the same objective yields the same graph (idempotent review).
//! - The graph still goes through `validate_graph` + owner acceptance.
//!   The planner never bypasses the C-05 single-writer command path.

use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
use dc_domain::work::{
    GraphCandidate, ObjectiveContract, WorkBounds, WorkKind, WorkUnitSpec,
    OutputContract, StopCondition, UnitRef, UnitStage,
};
use std::num::NonZeroU32;

use crate::error::CoreError;

/// Derive a minimal valid `GraphCandidate` from an objective.
///
/// The graph contains exactly one root unit of kind `Research` whose
/// acceptance criteria mirror the objective's completion criteria. This is
/// intentionally conservative: decomposition into sub-units is the owner's
/// decision at review time, not the planner's.
///
/// Returns `CoreError::InvalidCommand` if the objective is structurally
/// unplannable (empty completion criteria — already rejected upstream, but
/// rechecked here so the planner is total and safe to call directly).
pub fn plan_graph_from_objective(
    objective: &ObjectiveContract,
) -> Result<GraphCandidate, CoreError> {
    if objective.completion_criteria.is_empty() {
        return Err(CoreError::InvalidCommand(
            "cannot plan graph: objective has no completion criteria".into(),
        ));
    }

    let root_unit_id = derive_unit_id(&objective.scope.objective_id, 1);
    let revision = UnitRevision::new(1).expect("revision 1 is valid");

    let unit = WorkUnitSpec {
        unit: UnitRef {
            unit_id: root_unit_id,
            revision,
        },
        scope: objective.scope,
        parent: None,
        kind: WorkKind::Research,
        expected_output: OutputContract {
            export_name: "root_result".into(),
            schema_version: NonZeroU32::new(1).expect("schema 1"),
            fields: vec![],
        },
        obligation_ids: objective.obligations.iter().map(|o| o.obligation_id).collect(),
        acceptance_criteria: objective.completion_criteria.clone(),
        bounds: WorkBounds {
            max_attempts: NonZeroU32::new(3).expect("attempts 3"),
            max_steps: NonZeroU32::new(16).expect("steps 16"),
            stop_conditions: vec![StopCondition::OutputProduced],
        },
        exploration: None,
        stage: UnitStage::Proposed,
        accepted_result: None,
        row_version: RowVersion::new(1).expect("row 1"),
    };

    Ok(GraphCandidate {
        expected_graph_version: RowVersion::new(1).expect("row 1"),
        current_graph_version: RowVersion::new(1).expect("row 1"),
        roots: vec![],
        objectives: vec![objective.clone()],
        units: vec![unit],
        dependencies: vec![],
        links: vec![],
        publications: vec![],
        publication_states: vec![],
        publication_bindings: vec![],
        owner_acceptances: vec![],
    })
}

/// Derive a deterministic unit ID from the objective ID and a sequence number.
/// Derive a deterministic unit ID from the objective ID and a sequence number.
///
/// EntityId requires the UUIDv7 bit layout (version=7 at bits 76-79,
/// variant=0b10 at bits 62-63) and a non-nil value. We fold the objective's
/// text form with the sequence, then force the v7/variant bits so the result
/// is always a valid EntityId. The same objective always plans to the same
/// unit IDs, keeping review idempotent.
fn derive_unit_id(objective_id: &EntityId, seq: u64) -> EntityId {
    let text = objective_id.to_string();
    let base = u128::from_str_radix(&text, 16).expect("EntityId text is hex");
    // Fold in the sequence via a golden-ratio multiply-xor mix.
    let mut mixed = base ^ (seq as u128).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    // Force UUIDv7 layout: version nibble = 7, variant bits = 0b10.
    mixed = (mixed & !(0xfu128 << 76)) | (0x7u128 << 76);
    mixed = (mixed & !(0x3u128 << 62)) | (0x2u128 << 62);
    if mixed == 0 {
        mixed = (0x7u128 << 76) | (0x2u128 << 62) | 1; // minimal valid v7
    }
    EntityId::from_u128(mixed).expect("masked value is valid UUIDv7")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::ids::EntityId;
    use dc_domain::work::WorkScope;

    /// Build a valid UUIDv7 EntityId for tests: version=7, variant=0b10, seq fills the low bits.
    fn vid(n: u128) -> EntityId {
        let v = (0x7u128 << 76) | (0x2u128 << 62) | (n & ((1u128 << 62) - 1));
        EntityId::from_u128(v).expect("valid v7 test id")
    }

    fn scope() -> WorkScope {
        WorkScope {
            session_id: vid(1),
            work_id: vid(2),
            objective_id: vid(3),
            objective_revision: UnitRevision::new(1).unwrap(),
        }
    }

    fn objective() -> ObjectiveContract {
        ObjectiveContract {
            scope: scope(),
            statement: "Research topic X".into(),
            owner_event_ids: vec![],
            obligations: vec![],
            completion_criteria: vec![vid(4)],
            row_version: RowVersion::new(1).unwrap(),
        }
    }

    #[test]
    fn plans_single_root_unit() {
        let g = plan_graph_from_objective(&objective()).unwrap();
        assert_eq!(g.units.len(), 1);
        assert_eq!(g.units[0].stage, UnitStage::Proposed);
        assert!(g.units[0].parent.is_none());
        assert_eq!(g.objectives.len(), 1);
    }

    #[test]
    fn planning_is_deterministic() {
        let a = plan_graph_from_objective(&objective()).unwrap();
        let b = plan_graph_from_objective(&objective()).unwrap();
        assert_eq!(a.units[0].unit, b.units[0].unit);
    }

    #[test]
    fn rejects_empty_criteria() {
        let mut o = objective();
        o.completion_criteria.clear();
        assert!(plan_graph_from_objective(&o).is_err());
    }

    #[test]
    fn planned_graph_passes_validation() {
        let g = plan_graph_from_objective(&objective()).unwrap();
        // Reuse the core validator — the planned graph must be acceptable.
        assert!(crate::command::validate_graph(&g).is_ok());
    }
}
