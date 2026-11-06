//! Partial C-01 continuation handoff planner, NOT the resume authorization.
//! This pure plan hands a unit's continuation to a NEW attempt of the SAME unit:
//! it never revives the old attempt, never transfers the old send admission, and
//! never stores, executes or clears a blocking reason. This slice prepares a
//! retry after a confirmed failure (doc 16 §9.2, doc 14 §4.11). C-13
//! `prepare_resume`/`authorize_resume` still own the proposal and the owner
//! command; C-03 must re-check the attempt count and every row version
//! atomically at commit (per-attempt history lives in `unit_attempts`, so the
//! count is supplied by the caller, not stored on the in-memory attempt row).

use super::lifecycle::UNCLASSIFIED_ATTEMPT_OPERATIONS;
use super::*;
use crate::ids::RunGeneration;

/// Caller supplies the complete installation -> root -> unit gate chain plus the
/// exact previous unit/attempt basis and the durable per-unit attempt count read
/// from `unit_attempts`. `root_gates` must be read from the graph, never
/// substituted by caller unit gates. No slice is optional.
pub struct ContinuationContext<'a> {
    pub current_run: RunGeneration,
    pub installation_gates: &'a [BlockingReason],
    pub root_gates: &'a [BlockingReason],
    pub unit_gates: &'a [BlockingReason],
    /// Exact read of the unit row being continued; mismatch stops the plan.
    pub expected_unit_row_version: RowVersion,
    /// Exact read of the previous attempt row; mismatch stops the plan.
    pub expected_attempt_row_version: RowVersion,
    /// Number of attempts already recorded for this unit (`unit_attempts`
    /// count). The hard per-unit limit is checked against this; a restart does
    /// not reset it (doc 16 §9.2, doc 14 §4.11).
    pub attempts_used: u32,
}

/// Proposed replacements, not persisted rows or a started attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationPlan {
    unit: WorkUnitSpec,
    attempt: AttemptRecord,
    expected_basis: ExpectedLifecycleBasis,
    expected_attempts_used: u32,
    attempt_number: u32,
}

impl ContinuationPlan {
    pub fn unit(&self) -> &WorkUnitSpec {
        &self.unit
    }
    pub fn attempt(&self) -> &AttemptRecord {
        &self.attempt
    }
    pub fn expected_basis(&self) -> &ExpectedLifecycleBasis {
        &self.expected_basis
    }
    /// Commit must compare this count with the durable history in the same
    /// transaction as the old row basis and insertion of the new unique ID.
    pub fn expected_attempts_used(&self) -> u32 {
        self.expected_attempts_used
    }
    pub fn attempt_number(&self) -> u32 {
        self.attempt_number
    }
}

/// Hand continuation of `unit` to a new attempt after `previous` has ended.
///
/// The caller must supply this unit's most recent attempt and complete durable
/// count. C-03 must verify both at commit; this function cannot prove history
/// completeness or global ID uniqueness. Only `Blocked + Failed` is supported;
/// lost/cancelled attempts need a separate reconciled continuation contract.
/// The old attempt is never revived or prepared again; the new
/// attempt starts at `Prepared`. The retry budget must remain; reaching the
/// hard per-unit limit is refused (doc 16 §9.2). Any protective gate (stop,
/// recovery, integrity, resource, dependency, publication or unknown outcome)
/// blocks the handoff. No result, reservation or old send admission is moved
/// to the new attempt.
pub fn plan_continuation_new_attempt(
    unit: &WorkUnitSpec,
    previous: &AttemptRecord,
    new_attempt_id: EntityId,
    context: &ContinuationContext<'_>,
) -> Result<ContinuationPlan, WorkError> {
    // Basis: exact unit, exact previous attempt, same run generation.
    if previous.unit != unit.unit
        || previous.scope != unit.scope
        || previous.run_generation != context.current_run
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    if unit.row_version != context.expected_unit_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    if previous.row_version != context.expected_attempt_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    if new_attempt_id == previous.attempt_id {
        return Err(WorkError::Duplicate(new_attempt_id));
    }
    // Only the coherent confirmed-failure pair can enter this partial path.
    if previous.stage != AttemptStage::Failed || unit.stage != UnitStage::Blocked {
        return Err(WorkError::InvalidTransition);
    }
    if unit.accepted_result.is_some() {
        return Err(WorkError::InvalidField("unit_attempt_state"));
    }
    if context.attempts_used == 0 {
        return Err(WorkError::InvalidField("attempts_used"));
    }
    // Budget: max_attempts is the hard per-unit limit; a restart does not reset
    // it. The next attempt number is attempts_used + 1; reaching the limit is
    // refused as a blocked condition, not silently allowed (doc 16 §9.2).
    let limit = unit.bounds.max_attempts.get();
    let next_no = context
        .attempts_used
        .checked_add(1)
        .ok_or(WorkError::VersionOverflow)?;
    if next_no > limit {
        return Err(WorkError::Blocked(BlockingReason::ResourceUnavailable {}));
    }
    // Full installation -> root -> unit gate chain must be clear for a handoff.
    // Preparation shares the existing unclassified-attempt gate policy. It
    // does not actually launch an effect. This refuses to blindly
    // repeat an unknown external outcome: `UnknownOutcome` (and every other
    // protective gate) blocks the shared unclassified-attempt operation set.
    let gates = evaluate_gates(
        context
            .installation_gates
            .iter()
            .chain(context.root_gates.iter())
            .chain(context.unit_gates.iter()),
        UNCLASSIFIED_ATTEMPT_OPERATIONS,
    )?;
    if let Some(reason) = gates.blockers().iter().next() {
        return Err(WorkError::Blocked(reason.clone()));
    }
    let mut next_unit = unit.clone();
    // Ready has no current attempt. Creating Prepared uses the same coherent
    // Leased + Prepared pair as the existing Prepare command; Start rechecks gates.
    next_unit.stage = UnitStage::Leased;
    next_unit.row_version = next_row(unit.row_version)?;
    let attempt = AttemptRecord {
        attempt_id: new_attempt_id,
        scope: unit.scope,
        unit: unit.unit,
        run_generation: context.current_run,
        stage: AttemptStage::Prepared,
        row_version: RowVersion::MIN,
    };
    let expected_basis = ExpectedLifecycleBasis {
        scope: unit.scope,
        unit: unit.unit,
        unit_row_version: unit.row_version,
        attempt_id: Some(previous.attempt_id),
        attempt_row_version: Some(previous.row_version),
        run_generation: context.current_run,
    };
    Ok(ContinuationPlan {
        unit: next_unit,
        attempt,
        expected_basis,
        expected_attempts_used: context.attempts_used,
        attempt_number: next_no,
    })
}
