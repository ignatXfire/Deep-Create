//! Partial C-01 result/unit plan, NOT the complete verified-result transaction.
//! C-03 must compare packet, unit, attempt and dependency read bases and save
//! replacements plus the event atomically. AccountedResultPlan adds observed
//! usage read bindings; full storage, authority and provenance remain pending.

use super::*;
use crate::ids::RunGeneration;
use crate::usage_binding::UsageBinding;

/// Terminal result/unit replacements bound to already-accounted usage reads.
/// Still not a full C-03 transaction or proof of current authority/manifest scope.
#[derive(Debug, Clone)]
pub struct AccountedResultPlan {
    transition: ResultTransitionPlan,
    usage: UsageBinding,
}

impl AccountedResultPlan {
    pub fn transition(&self) -> &ResultTransitionPlan {
        &self.transition
    }
    pub fn usage(&self) -> &UsageBinding {
        &self.usage
    }
}

/// Bind acceptance/rejection to facts of this exact producing attempt. Never
/// applies observations again: costs survive even when content is rejected.
pub fn plan_accounted_result_transition(
    packet: &ResultPacket,
    target: ResultStage,
    verification: &VerificationRecord,
    context: &ResultTransitionContext<'_>,
    usage: &UsageBinding,
    event_id: EntityId,
) -> Result<AccountedResultPlan, WorkError> {
    if !matches!(target, ResultStage::Accepted | ResultStage::Rejected) {
        return Err(WorkError::InvalidTransition);
    }
    if usage.attempt_id() != packet.attempt_id {
        return Err(WorkError::ResultBasisMismatch);
    }
    // A late over-bound bill remains a fact for rejection, but cannot endorse
    // accepted content. Binding the fact itself must not discard the breach.
    if target == ResultStage::Accepted && usage.positions().iter().any(|p| p.bound_breached()) {
        return Err(WorkError::Blocked(BlockingReason::ResourceUnavailable {}));
    }
    let transition = plan_result_transition(packet, target, Some(verification), context, event_id)?;
    Ok(AccountedResultPlan {
        transition,
        usage: usage.clone(),
    })
}

/// Caller supplies authenticated current rows and the full protected gate chain.
/// Root gates are read from the graph, never substituted by caller gate lists.
pub struct ResultTransitionContext<'a> {
    pub result_view: &'a ResultCommitView<'a>,
    pub state: &'a UnitAttemptState,
    pub expected: &'a ExpectedLifecycleBasis,
    pub current_run: RunGeneration,
    pub installation_gates: &'a [BlockingReason],
    pub unit_gates: &'a [BlockingReason],
}

/// Proposed event, not a persisted receipt or authority-bearing input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultTransitionEvent {
    pub event_id: EntityId,
    pub result_id: EntityId,
    pub attempt_id: EntityId,
    pub scope: WorkScope,
    pub unit: UnitRef,
    pub result_from: ResultStage,
    pub result_to: ResultStage,
    pub unit_from: UnitStage,
    pub unit_to: UnitStage,
    pub verification_id: Option<EntityId>,
}

/// Opaque replacements with exact row preconditions. The Submitted attempt is
/// a read precondition, NOT an unchanged row to write over a concurrent update.
#[derive(Debug, Clone)]
pub struct ResultTransitionPlan {
    packet: ResultPacket,
    unit: WorkUnitSpec,
    basis: ExpectedLifecycleBasis,
    expected_result_row_version: RowVersion,
    event: ResultTransitionEvent,
}

impl ResultTransitionPlan {
    pub fn packet(&self) -> &ResultPacket {
        &self.packet
    }
    pub fn unit(&self) -> &WorkUnitSpec {
        &self.unit
    }
    pub fn expected_basis(&self) -> &ExpectedLifecycleBasis {
        &self.basis
    }
    pub fn expected_result_row_version(&self) -> RowVersion {
        self.expected_result_row_version
    }
    pub fn event(&self) -> &ResultTransitionEvent {
        &self.event
    }
}

/// Couple the existing result validator to its exact matching unit replacement.
/// Any error leaves all inputs unchanged. Rejection/staleness preserve facts;
/// verification/acceptance require clear installation, root AND unit gates.
pub fn plan_result_transition(
    packet: &ResultPacket,
    target: ResultStage,
    verification: Option<&VerificationRecord>,
    context: &ResultTransitionContext<'_>,
    event_id: EntityId,
) -> Result<ResultTransitionPlan, WorkError> {
    let view = context.result_view;
    let state = context.state;
    let basis = context.expected;
    if view.graph.unit(packet.unit) != Some(&state.unit)
        || basis.scope != packet.scope
        || basis.unit != packet.unit
        || state.unit.scope != packet.scope
        || basis.unit_row_version != view.expected_unit_row_version
        || basis.attempt_id != Some(packet.attempt_id)
        || basis.run_generation != context.current_run
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    if state.unit.row_version != basis.unit_row_version
        || packet.row_version != view.expected_result_row_version
    {
        return Err(WorkError::RowVersionConflict);
    }
    if state.unit.accepted_result.is_some() {
        return Err(WorkError::InvalidTransition);
    }
    let attempt = state
        .attempt
        .as_ref()
        .ok_or(WorkError::ResultBasisMismatch)?;
    if attempt.attempt_id != packet.attempt_id
        || attempt.scope != packet.scope
        || attempt.unit != packet.unit
        || attempt.stage != AttemptStage::Submitted
        || attempt.run_generation != context.current_run
        || Some(attempt.row_version) != basis.attempt_row_version
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    // Do not turn an unchecked, irrelevant record into apparent event evidence.
    if verification.is_some() && !matches!(target, ResultStage::Accepted | ResultStage::Rejected) {
        return Err(WorkError::VerificationMismatch);
    }
    if matches!(target, ResultStage::Verifying | ResultStage::Accepted) {
        let root = view
            .graph
            .candidate()
            .roots
            .iter()
            .find(|r| r.work_id == packet.scope.work_id)
            .ok_or(WorkError::ResultBasisMismatch)?;
        let gates = evaluate_gates(
            context
                .installation_gates
                .iter()
                .chain(root.blocking_reasons.iter())
                .chain(context.unit_gates.iter()),
            &[GateOperation::SubmitResult],
        )?;
        if let Some(reason) = gates.blockers().iter().next() {
            return Err(WorkError::Blocked(reason.clone()));
        }
    }
    // This validates legal source stages, result identity, freshness and proof.
    let next_packet = transition_result(packet, target, verification, view)?;
    let unit_to = match target {
        ResultStage::Verifying => UnitStage::Verifying,
        ResultStage::Accepted => UnitStage::Accepted,
        ResultStage::Rejected => UnitStage::Rejected,
        ResultStage::Stale => UnitStage::Stale,
        ResultStage::Submitted => return Err(WorkError::InvalidTransition),
    };
    let mut next_unit = state.unit.clone();
    next_unit.stage = unit_to;
    next_unit.accepted_result = (target == ResultStage::Accepted).then_some(AcceptedResultRef {
        result_id: next_packet.result_id,
        snapshot_id: next_packet.snapshot.snapshot_id,
        fingerprint: next_packet.fingerprint,
        result_row_version: next_packet.row_version,
    });
    next_unit.row_version = state
        .unit
        .row_version
        .get()
        .checked_add(1)
        .and_then(RowVersion::new)
        .ok_or(WorkError::VersionOverflow)?;
    let event = ResultTransitionEvent {
        event_id,
        result_id: packet.result_id,
        attempt_id: packet.attempt_id,
        scope: packet.scope,
        unit: packet.unit,
        result_from: packet.stage,
        result_to: target,
        unit_from: state.unit.stage,
        unit_to,
        verification_id: verification.map(|v| v.verification_id),
    };
    Ok(ResultTransitionPlan {
        packet: next_packet,
        unit: next_unit,
        basis: basis.clone(),
        expected_result_row_version: view.expected_result_row_version,
        event,
    })
}
