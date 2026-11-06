//! A bounded C-01 lifecycle planner, not the complete C-05 command dispatcher.
//! Plans contain proposed replacements/events. Only C-03 may commit them.

use super::*;
use crate::ids::RunGeneration;

/// Closed operation classes for the C-01 gate check, not capability requests.
/// The controller must classify the real operation; a worker's label is not proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GateOperation {
    Read,
    Reason,
    Plan,
    Effect,
    Cancel,
    /// Only pre-registered, bounded local recovery/stop diagnostics, not old units.
    TrustedLocalDiagnosticRead,
    /// A service-path reconciliation step. Any network/read/spend still needs its
    /// own exact authority and admission checks; this class cannot replace them.
    Reconcile,
    /// Record an authenticated fact without accepting its semantic result.
    ObserveFact,
    /// Receive a trusted owner decision, not execute everything it requests.
    ReceiveOwnerDecision,
    /// Promote a running attempt's packet into the semantic verification path.
    SubmitResult,
    /// Apply the exact final owner decision after candidate/freshness validation.
    AcceptCompletion,
}

#[derive(Clone, Copy)]
struct GatePolicy {
    read: bool,
    reason: bool,
    plan: bool,
    effect: bool,
    cancel: bool,
    diagnostic_read: bool,
    reconcile: bool,
    observation: bool,
    owner_decision: bool,
    result_submission: bool,
    completion_acceptance: bool,
}

impl GatePolicy {
    const PROTECTIVE: Self = Self {
        read: false,
        reason: false,
        plan: false,
        effect: false,
        cancel: true,
        diagnostic_read: true,
        reconcile: true,
        observation: true,
        owner_decision: true,
        result_submission: false,
        completion_acceptance: false,
    };

    fn for_reason(reason: &BlockingReason) -> Self {
        // Exhaustive on both axes: a new gate or operation needs an explicit rule.
        match reason {
            BlockingReason::Recovery {}
            | BlockingReason::IntegrityFailure {}
            | BlockingReason::ResourceUnavailable {}
            | BlockingReason::StopRequested {} => Self::PROTECTIVE,
            BlockingReason::AwaitingOwner {} => Self {
                read: true,
                reason: true,
                plan: true,
                completion_acceptance: true,
                ..Self::PROTECTIVE
            },
            BlockingReason::UnknownOutcome {} | BlockingReason::OwnerRejectionPending { .. } => {
                Self {
                    read: true,
                    reason: true,
                    plan: true,
                    ..Self::PROTECTIVE
                }
            }
            // These gates concern this precise dependent unit, not unrelated
            // units. Missing/invalid required input is not a usable read basis.
            BlockingReason::AwaitingPublication { .. }
            | BlockingReason::PublicationUnresolved { .. }
            | BlockingReason::PublicationWithdrawn { .. }
            | BlockingReason::OwnerAcceptancePending { .. }
            | BlockingReason::ConditionUnknown { .. }
            | BlockingReason::UpstreamCancelled { .. }
            | BlockingReason::DependencyPending { .. } => Self::PROTECTIVE,
        }
    }

    fn does_not_block(self, operation: GateOperation) -> bool {
        match operation {
            GateOperation::Read => self.read,
            GateOperation::Reason => self.reason,
            GateOperation::Plan => self.plan,
            GateOperation::Effect => self.effect,
            GateOperation::Cancel => self.cancel,
            GateOperation::TrustedLocalDiagnosticRead => self.diagnostic_read,
            GateOperation::Reconcile => self.reconcile,
            GateOperation::ObserveFact => self.observation,
            GateOperation::ReceiveOwnerDecision => self.owner_decision,
            GateOperation::SubmitResult => self.result_submission,
            GateOperation::AcceptCompletion => self.completion_acceptance,
        }
    }
}

/// Negative gate information only. Even an empty blocker set is not authority,
/// a reservation, proof of storage health, a commit receipt, or permission to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateAssessment {
    blockers: BTreeSet<BlockingReason>,
}

impl GateAssessment {
    pub fn blockers(&self) -> &BTreeSet<BlockingReason> {
        &self.blockers
    }

    pub fn is_clear(&self) -> bool {
        self.blockers.is_empty()
    }

    fn require_clear(self) -> Result<(), WorkError> {
        match self.blockers.into_iter().next() {
            Some(reason) => Err(WorkError::Blocked(reason)),
            None => Ok(()),
        }
    }
}

/// Conjunctive check of every supplied gate and every required operation class.
/// Pass the complete installation -> root -> unit chain (chained iterators are
/// supported). C-01 cannot establish that the caller omitted no protected scope.
/// An empty operation set fails closed instead of becoming an unclassified pass.
pub fn evaluate_gates<'a>(
    gates: impl IntoIterator<Item = &'a BlockingReason>,
    operations: &[GateOperation],
) -> Result<GateAssessment, WorkError> {
    if operations.is_empty() {
        return Err(WorkError::InvalidField("gate_operations"));
    }
    let blockers = gates
        .into_iter()
        .filter(|reason| {
            let policy = GatePolicy::for_reason(reason);
            operations
                .iter()
                .any(|operation| !policy.does_not_block(*operation))
        })
        .cloned()
        .collect();
    Ok(GateAssessment { blockers })
}

// WorkKind describes the expected result, not a proven effect-free execution
// profile. Existing generic Prepare/Start must remain conservative until C-05
// supplies exact classified actions. A Planning label cannot bypass Effect.
pub(super) const UNCLASSIFIED_ATTEMPT_OPERATIONS: &[GateOperation] = &[
    GateOperation::Read,
    GateOperation::Reason,
    GateOperation::Plan,
    GateOperation::Effect,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStage {
    Prepared,
    Running,
    Submitted,
    Failed,
    Lost,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptRecord {
    pub attempt_id: EntityId,
    pub scope: WorkScope,
    pub unit: UnitRef,
    pub run_generation: RunGeneration,
    pub stage: AttemptStage,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitAttemptState {
    pub unit: WorkUnitSpec,
    pub attempt: Option<AttemptRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedLifecycleBasis {
    pub scope: WorkScope,
    pub unit: UnitRef,
    pub unit_row_version: RowVersion,
    pub attempt_id: Option<EntityId>,
    pub attempt_row_version: Option<RowVersion>,
    pub run_generation: RunGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UnitAttemptCommand {
    Prepare { new_attempt_id: EntityId },
    Start {},
    ObserveSubmission { result_id: EntityId },
    RequestCancellation {},
    ConfirmCancellation {},
    ObserveLost { observation_id: EntityId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    AttemptPrepared {
        attempt_id: EntityId,
    },
    AttemptStarted {
        attempt_id: EntityId,
    },
    ResultSubmitted {
        attempt_id: EntityId,
        result_id: EntityId,
    },
    CancellationRequested {
        unit: UnitRef,
    },
    AttemptCancelled {
        attempt_id: EntityId,
    },
    UnitCancelled {
        unit: UnitRef,
    },
    AttemptLost {
        attempt_id: EntityId,
        observation_id: EntityId,
    },
    LateObservation {
        attempt_id: EntityId,
        observation_id: EntityId,
    },
    RootStageChanged {
        work_id: EntityId,
        from: RootStage,
        to: RootStage,
    },
    OwnerCompletionAccepted {
        work_id: EntityId,
        candidate_id: EntityId,
        owner_event_id: EntityId,
        final_verification_id: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitLifecyclePlan {
    replacement: Option<UnitAttemptState>,
    events: Vec<LifecycleEvent>,
    expected: ExpectedLifecycleBasis,
}

impl UnitLifecyclePlan {
    pub fn replacement(&self) -> Option<&UnitAttemptState> {
        self.replacement.as_ref()
    }
    pub fn events(&self) -> &[LifecycleEvent] {
        &self.events
    }
    pub fn expected(&self) -> &ExpectedLifecycleBasis {
        &self.expected
    }
    pub fn observation_only(&self) -> bool {
        self.replacement.is_none()
    }
}

pub(super) fn next_row(row: RowVersion) -> Result<RowVersion, WorkError> {
    row.get()
        .checked_add(1)
        .and_then(RowVersion::new)
        .ok_or(WorkError::VersionOverflow)
}

/// Gates come from the complete installation -> root -> unit chain. This pure
/// planner does not authenticate a worker or infer rights from a prepared lease.
pub fn plan_unit_attempt_transition(
    state: &UnitAttemptState,
    expected: &ExpectedLifecycleBasis,
    current_scope: WorkScope,
    current_run: RunGeneration,
    gates: &BTreeSet<BlockingReason>,
    command: &UnitAttemptCommand,
) -> Result<UnitLifecyclePlan, WorkError> {
    let attempt_stage = state.attempt.as_ref().map(|a| a.stage);
    let coherent = match state.unit.stage {
        UnitStage::Proposed | UnitStage::Ready => attempt_stage.is_none(),
        UnitStage::Leased => attempt_stage == Some(AttemptStage::Prepared),
        UnitStage::Running => attempt_stage == Some(AttemptStage::Running),
        UnitStage::ResultSubmitted
        | UnitStage::Verifying
        | UnitStage::Accepted
        | UnitStage::Rejected
        | UnitStage::Stale => attempt_stage == Some(AttemptStage::Submitted),
        UnitStage::Cancelling => matches!(
            attempt_stage,
            None | Some(AttemptStage::Prepared | AttemptStage::Running)
        ),
        UnitStage::Cancelled => matches!(attempt_stage, None | Some(AttemptStage::Cancelled)),
        UnitStage::Lost => attempt_stage == Some(AttemptStage::Lost),
        UnitStage::Blocked => matches!(
            attempt_stage,
            None | Some(AttemptStage::Prepared | AttemptStage::Running | AttemptStage::Failed)
        ),
    };
    if !coherent
        || (state.unit.stage == UnitStage::Accepted) != state.unit.accepted_result.is_some()
    {
        return Err(WorkError::InvalidField("unit_attempt_state"));
    }
    if state.unit.scope != current_scope
        || expected.scope != current_scope
        || expected.unit != state.unit.unit
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    if expected.unit_row_version != state.unit.row_version
        || expected.attempt_row_version != state.attempt.as_ref().map(|a| a.row_version)
    {
        return Err(WorkError::RowVersionConflict);
    }
    if expected.attempt_id != state.attempt.as_ref().map(|a| a.attempt_id) {
        return Err(WorkError::ResultBasisMismatch);
    }
    if let Some(attempt) = &state.attempt
        && (attempt.unit != state.unit.unit
            || attempt.scope != current_scope
            || attempt.run_generation != expected.run_generation)
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    let is_observation = matches!(
        command,
        UnitAttemptCommand::ObserveSubmission { .. } | UnitAttemptCommand::ObserveLost { .. }
    );
    if expected.run_generation > current_run {
        return Err(WorkError::ResultBasisMismatch);
    }
    let operations =
        match command {
            UnitAttemptCommand::Prepare { .. } | UnitAttemptCommand::Start {} => {
                UNCLASSIFIED_ATTEMPT_OPERATIONS
            }
            UnitAttemptCommand::ObserveSubmission { .. }
            | UnitAttemptCommand::ObserveLost { .. } => &[GateOperation::ObserveFact],
            UnitAttemptCommand::RequestCancellation {}
            | UnitAttemptCommand::ConfirmCancellation {} => &[GateOperation::Cancel],
        };
    evaluate_gates(gates, operations)?.require_clear()?;
    if expected.run_generation < current_run {
        if is_observation {
            let attempt_id = expected.attempt_id.ok_or(WorkError::ResultBasisMismatch)?;
            let observation_id = match command {
                UnitAttemptCommand::ObserveSubmission { result_id } => *result_id,
                UnitAttemptCommand::ObserveLost { observation_id } => *observation_id,
                _ => return Err(WorkError::InvalidTransition),
            };
            return Ok(UnitLifecyclePlan {
                replacement: None,
                events: vec![LifecycleEvent::LateObservation {
                    attempt_id,
                    observation_id,
                }],
                expected: expected.clone(),
            });
        }
        return Err(WorkError::ResultBasisMismatch);
    }
    let mut next = state.clone();
    let mut events = Vec::new();
    match command {
        UnitAttemptCommand::Prepare { new_attempt_id } => {
            if state.unit.stage != UnitStage::Ready || state.attempt.is_some() {
                return Err(WorkError::InvalidTransition);
            }
            next.unit.stage = UnitStage::Leased;
            next.attempt = Some(AttemptRecord {
                attempt_id: *new_attempt_id,
                scope: current_scope,
                unit: state.unit.unit,
                run_generation: current_run,
                stage: AttemptStage::Prepared,
                row_version: RowVersion::MIN,
            });
            events.push(LifecycleEvent::AttemptPrepared {
                attempt_id: *new_attempt_id,
            });
        }
        UnitAttemptCommand::RequestCancellation {} => {
            if !matches!(
                state.unit.stage,
                UnitStage::Ready | UnitStage::Leased | UnitStage::Running
            ) {
                return Err(WorkError::InvalidTransition);
            }
            next.unit.stage = UnitStage::Cancelling;
            events.push(LifecycleEvent::CancellationRequested {
                unit: state.unit.unit,
            });
        }
        UnitAttemptCommand::ConfirmCancellation {} if state.attempt.is_none() => {
            if state.unit.stage != UnitStage::Cancelling {
                return Err(WorkError::InvalidTransition);
            }
            next.unit.stage = UnitStage::Cancelled;
            events.push(LifecycleEvent::UnitCancelled {
                unit: state.unit.unit,
            });
        }
        _ => {
            let attempt = next
                .attempt
                .as_mut()
                .ok_or(WorkError::ResultBasisMismatch)?;
            match command {
                UnitAttemptCommand::Start {} => {
                    if state.unit.stage != UnitStage::Leased
                        || attempt.stage != AttemptStage::Prepared
                    {
                        return Err(WorkError::InvalidTransition);
                    }
                    next.unit.stage = UnitStage::Running;
                    attempt.stage = AttemptStage::Running;
                    events.push(LifecycleEvent::AttemptStarted {
                        attempt_id: attempt.attempt_id,
                    });
                }
                UnitAttemptCommand::ObserveSubmission { result_id } => {
                    if state.unit.stage != UnitStage::Running
                        || attempt.stage != AttemptStage::Running
                        || !evaluate_gates(gates, &[GateOperation::SubmitResult])?.is_clear()
                    {
                        return Ok(UnitLifecyclePlan {
                            replacement: None,
                            events: vec![LifecycleEvent::LateObservation {
                                attempt_id: attempt.attempt_id,
                                observation_id: *result_id,
                            }],
                            expected: expected.clone(),
                        });
                    }
                    next.unit.stage = UnitStage::ResultSubmitted;
                    attempt.stage = AttemptStage::Submitted;
                    events.push(LifecycleEvent::ResultSubmitted {
                        attempt_id: attempt.attempt_id,
                        result_id: *result_id,
                    });
                }
                UnitAttemptCommand::ConfirmCancellation {} => {
                    if state.unit.stage != UnitStage::Cancelling
                        || !matches!(
                            attempt.stage,
                            AttemptStage::Prepared | AttemptStage::Running
                        )
                    {
                        return Err(WorkError::InvalidTransition);
                    }
                    next.unit.stage = UnitStage::Cancelled;
                    attempt.stage = AttemptStage::Cancelled;
                    events.push(LifecycleEvent::AttemptCancelled {
                        attempt_id: attempt.attempt_id,
                    });
                }
                UnitAttemptCommand::ObserveLost { observation_id } => {
                    if state.unit.stage != UnitStage::Running
                        || attempt.stage != AttemptStage::Running
                    {
                        return Ok(UnitLifecyclePlan {
                            replacement: None,
                            events: vec![LifecycleEvent::LateObservation {
                                attempt_id: attempt.attempt_id,
                                observation_id: *observation_id,
                            }],
                            expected: expected.clone(),
                        });
                    }
                    next.unit.stage = UnitStage::Lost;
                    attempt.stage = AttemptStage::Lost;
                    events.push(LifecycleEvent::AttemptLost {
                        attempt_id: attempt.attempt_id,
                        observation_id: *observation_id,
                    });
                }
                _ => return Err(WorkError::InvalidTransition),
            }
            attempt.row_version = next_row(attempt.row_version)?;
        }
    }
    next.unit.row_version = next_row(state.unit.row_version)?;
    Ok(UnitLifecyclePlan {
        replacement: Some(next),
        events,
        expected: expected.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalReviewEvidence {
    pub verification_id: EntityId,
    pub candidate_id: EntityId,
    pub scope: WorkScope,
    pub reviewed_results: Vec<CompletionBasis>,
    pub verdict: VerificationVerdict,
    /// The source verification record that proves qualified review.
    /// Without this, final review is just "someone said PASS".
    pub source_verification: Option<VerificationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedCompletion {
    candidate: CompletionCandidate,
    verification_id: EntityId,
}

impl ReviewedCompletion {
    pub fn candidate(&self) -> &CompletionCandidate {
        &self.candidate
    }

    pub fn verification_id(&self) -> EntityId {
        self.verification_id
    }
}

pub fn validate_final_review(
    candidate: &CompletionCandidate,
    evidence: &FinalReviewEvidence,
    unit: &WorkUnitSpec,
    _objective: &ObjectiveContract,
) -> Result<ReviewedCompletion, WorkError> {
    if evidence.candidate_id != candidate.candidate_id()
        || evidence.scope != candidate.scope()
        || evidence.reviewed_results != candidate.results()
        || evidence.verdict != VerificationVerdict::Pass
    {
        return Err(WorkError::VerificationMismatch);
    }
    
    // Final review must come from a qualified C-08 verification record.
    // Without this, "someone with the right ID said PASS" is enough.
    if let Some(verification) = &evidence.source_verification {
        // Verify the record is for the correct unit
        if verification.unit != unit.unit {
            return Err(WorkError::VerificationMismatch);
        }
        
        // Verify the record is for the correct scope
        if verification.evidence.scope != unit.scope {
            return Err(WorkError::VerificationMismatch);
        }
        
        // Verify method qualification
        let evidence_data = &verification.evidence;
        if evidence_data.work_kind != unit.kind {
            return Err(WorkError::VerificationMismatch);
        }
        
        // Check level requirements
        let minimum = match unit.kind {
            WorkKind::DeterministicCheck | WorkKind::Verification => VerificationLevel::MachineOnly,
            WorkKind::Research | WorkKind::CreativeExploration | WorkKind::Planning | WorkKind::Summary => {
                VerificationLevel::IndependentReview
            }
            WorkKind::Synthesis | WorkKind::PreflightAnalysis | WorkKind::LocalChangeProposal => {
                VerificationLevel::DiverseChallenge
            }
            WorkKind::ModelAdaptation => VerificationLevel::ProtectedAdmission,
        };
        
        if evidence_data.level < minimum {
            return Err(WorkError::VerificationMismatch);
        }
        
        // Verify the verdict matches
        if verification.verdict != VerificationVerdict::Pass {
            return Err(WorkError::VerificationMismatch);
        }
    } else {
        // No source verification = no proof of qualified review
        return Err(WorkError::VerificationMismatch);
    }
    
    Ok(ReviewedCompletion {
        candidate: candidate.clone(),
        verification_id: evidence.verification_id,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootLifecycleState {
    pub root: RootWork,
    pub scope: WorkScope,
    pub candidate: Option<CompletionCandidate>,
    pub final_review: Option<ReviewedCompletion>,
    pub pending_owner_rejection: Option<PendingOwnerRejection>,
}

pub enum RootLifecycleCommand<'a> {
    Start(&'a ValidatedObjective<'a>),
    PresentCandidate(&'a CompletionCandidate),
    BeginFinalReview,
    CompleteFinalReview(&'a ReviewedCompletion),
    AcceptByOwner(&'a OwnerAcceptedCandidate),
}

pub struct CompletionCheckView<'a> {
    pub graph: &'a ValidatedGraph<'a>,
    pub results: &'a [AcceptedResultEvidence<'a>],
    pub current_rows: &'a [VersionedDependency],
}

fn check_fresh_completion(
    candidate: &CompletionCandidate,
    view: Option<&CompletionCheckView<'_>>,
) -> Result<(), WorkError> {
    let view = view.ok_or(WorkError::ResultBasisMismatch)?;
    let current = derive_completion_candidate(
        candidate.candidate_id(),
        candidate.scope().objective_id,
        view.graph,
        view.results,
        view.current_rows,
    )
    .map_err(|_| WorkError::ResultBasisMismatch)?;
    if &current != candidate {
        return Err(WorkError::ResultBasisMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootLifecyclePlan {
    replacement: RootLifecycleState,
    expected_row_version: RowVersion,
    event: LifecycleEvent,
}

impl RootLifecyclePlan {
    pub fn replacement(&self) -> &RootLifecycleState {
        &self.replacement
    }
    pub fn expected_row_version(&self) -> RowVersion {
        self.expected_row_version
    }
    pub fn event(&self) -> &LifecycleEvent {
        &self.event
    }
}

pub(super) fn check_root_basis(
    state: &RootLifecycleState,
    expected_row_version: RowVersion,
    current_scope: WorkScope,
) -> Result<(), WorkError> {
    if state.root.row_version != expected_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    if state.scope != current_scope
        || state.root.active_objective_id != current_scope.objective_id
        || state.root.work_id != current_scope.work_id
        || state.root.session_id != current_scope.session_id
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    let coherent = match state.root.stage {
        RootStage::Draft | RootStage::Executing => {
            state.candidate.is_none() && state.final_review.is_none()
        }
        RootStage::CompletionCandidate | RootStage::Verifying => {
            state
                .candidate
                .as_ref()
                .is_some_and(|c| c.scope() == current_scope)
                && state.final_review.is_none()
        }
        RootStage::AwaitingOwnerAcceptance | RootStage::OwnerAccepted => {
            state.candidate.as_ref().is_some_and(|c| {
                c.scope() == current_scope
                    && state
                        .final_review
                        .as_ref()
                        .is_some_and(|review| &review.candidate == c)
            })
        }
    };
    if !coherent {
        return Err(WorkError::ResultBasisMismatch);
    }
    if let Some(pending) = &state.pending_owner_rejection
        && (state.root.stage != RootStage::AwaitingOwnerAcceptance
            || state.final_review.as_ref() != Some(pending.completion())
            || pending.decision().scope != current_scope
            || pending.decision().candidate_id != pending.completion().candidate().candidate_id()
            || pending.decision().final_verification_id != pending.completion().verification_id()
            || !state
                .root
                .blocking_reasons
                .contains(&BlockingReason::OwnerRejectionPending {
                    owner_event_id: pending.decision().owner_event_id,
                }))
    {
        return Err(WorkError::OwnerRejectionMismatch);
    }
    Ok(())
}

/// Scope and versions must be rechecked by C-03 at commit. Freshness of a
/// completion candidate must also be rederived in that coherent commit view.
/// `installation_gates` are combined with, never substituted for, this root's
/// own gates. Empty gate results do not validate authority or storage health.
pub fn plan_root_transition(
    state: &RootLifecycleState,
    expected_row_version: RowVersion,
    current_scope: WorkScope,
    completion_view: Option<&CompletionCheckView<'_>>,
    installation_gates: &BTreeSet<BlockingReason>,
    command: RootLifecycleCommand<'_>,
) -> Result<RootLifecyclePlan, WorkError> {
    check_root_basis(state, expected_row_version, current_scope)?;
    let operations = match &command {
        RootLifecycleCommand::Start(_) => UNCLASSIFIED_ATTEMPT_OPERATIONS,
        RootLifecycleCommand::PresentCandidate(_)
        | RootLifecycleCommand::BeginFinalReview
        | RootLifecycleCommand::CompleteFinalReview(_) => &[GateOperation::SubmitResult],
        RootLifecycleCommand::AcceptByOwner(_) => &[
            GateOperation::ReceiveOwnerDecision,
            GateOperation::AcceptCompletion,
        ],
    };
    evaluate_gates(
        installation_gates
            .iter()
            .chain(&state.root.blocking_reasons),
        operations,
    )?
    .require_clear()?;
    let mut next = state.clone();
    let mut acceptance_event = None;
    match (state.root.stage, command) {
        (RootStage::Draft, RootLifecycleCommand::Start(objective))
            if objective.contract().scope == current_scope =>
        {
            next.root.stage = RootStage::Executing
        }
        (RootStage::Executing, RootLifecycleCommand::PresentCandidate(candidate))
            if candidate.scope() == current_scope =>
        {
            check_fresh_completion(candidate, completion_view)?;
            next.root.stage = RootStage::CompletionCandidate;
            next.candidate = Some(candidate.clone());
            next.final_review = None;
        }
        (RootStage::CompletionCandidate, RootLifecycleCommand::BeginFinalReview)
            if state.candidate.is_some() =>
        {
            check_fresh_completion(
                state
                    .candidate
                    .as_ref()
                    .ok_or(WorkError::ResultBasisMismatch)?,
                completion_view,
            )?;
            next.root.stage = RootStage::Verifying;
        }
        (RootStage::Verifying, RootLifecycleCommand::CompleteFinalReview(review))
            if state.candidate.as_ref() == Some(&review.candidate) =>
        {
            check_fresh_completion(&review.candidate, completion_view)?;
            next.root.stage = RootStage::AwaitingOwnerAcceptance;
            next.final_review = Some(review.clone());
        }
        (RootStage::AwaitingOwnerAcceptance, RootLifecycleCommand::AcceptByOwner(accepted)) => {
            let review = state
                .final_review
                .as_ref()
                .ok_or(WorkError::OwnerAcceptanceMismatch)?;
            check_fresh_completion(&review.candidate, completion_view)?;
            if accepted.candidate_id != review.candidate.candidate_id()
                || accepted.scope != current_scope
                || accepted.final_verification_id != review.verification_id
            {
                return Err(WorkError::OwnerAcceptanceMismatch);
            }
            next.root.stage = RootStage::OwnerAccepted;
            acceptance_event = Some(LifecycleEvent::OwnerCompletionAccepted {
                work_id: state.root.work_id,
                candidate_id: accepted.candidate_id,
                owner_event_id: accepted.owner_event_id,
                final_verification_id: accepted.final_verification_id,
            });
        }
        _ => return Err(WorkError::InvalidTransition),
    }
    next.root.row_version = next_row(state.root.row_version)?;
    let event = acceptance_event.unwrap_or(LifecycleEvent::RootStageChanged {
        work_id: state.root.work_id,
        from: state.root.stage,
        to: next.root.stage,
    });
    Ok(RootLifecyclePlan {
        replacement: next,
        expected_row_version,
        event,
    })
}
