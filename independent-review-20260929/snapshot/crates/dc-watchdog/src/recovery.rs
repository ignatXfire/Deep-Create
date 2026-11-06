//! C-10 recovery/watchdog evidence handling: external-action outcomes.
//!
//! The watchdog owns the protective dispatch facts that survive an unavailable
//! main database (contract 18 §1): receipts of fencing, admission and
//! observation stored in `authority.anchor`. This module supplies the typed,
//! deterministic half of that evidence — the three-way external outcome and
//! the reconcile-or-refuse transition — without touching the SQLite anchor or
//! the live Windows gate. Persistence and the real reconciliation handler
//! through C-09/C-12 remain their own integration steps.
//!
//! Contract 14 §7.4 fixes the only legal external-action path:
//!
//! ```text
//! executing -> succeeded | failed_confirmed | outcome_unknown
//! outcome_unknown -> reconciling -> succeeded | failed_confirmed | unresolved
//! ```
//!
//! `outcome_unknown` is a first-class evidence state, not an ordinary `Err`
//! (contract 19 §4): it keeps the held cost visible and forbids a blind repeat
//! until a permitted reconciliation proves the final result (contract 21 И-05,
//! AC-I05: a lost response after dispatch must not cause a second effect).

use std::collections::BTreeMap;

use dc_domain::ids::{AuthorityEpoch, EntityId, RowVersion, RunGeneration};
use dc_domain::permission::{ActionKind, AuthoritySnapshot, FenceScope};
use thiserror::Error;

/// The three-way external outcome from contract 18 §8 / §9.1 step 10. A result
/// is either a confirmed success, a confirmed failure, or still unknown — there
/// is no fourth "probably fine" state to keep a visible, held-cost outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalOutcome {
    Succeeded,
    FailedConfirmed,
    OutcomeUnknown,
}

/// The external-action stage for one dispatched intent (contract 14 §7.4).
/// `DispatchStarted` and `Reconciling` are transient entry states; the rest are
/// the observable positions of the evidence record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalActionStage {
    /// `action_dispatch_started`: admitted before OS handoff, no outcome yet.
    DispatchStarted,
    /// `action_observed` confirmed success.
    Succeeded,
    /// `action_observed` confirmed failure.
    FailedConfirmed,
    /// `action_outcome_unknown`: blind repeat is forbidden.
    OutcomeUnknown,
    /// `action_reconciliation_started`: a permitted reconciliation is running.
    Reconciling,
    /// The reconciled outcome still cannot be proven (`unresolved`).
    Unresolved,
}

impl ExternalActionStage {
    /// Project onto the three-way external outcome. A dispatch without a proven
    /// observation, a pending reconciliation and an unresolved reconciliation
    /// all project to `OutcomeUnknown` — never to a fabricated success/failure.
    pub fn outcome(self) -> ExternalOutcome {
        match self {
            ExternalActionStage::Succeeded => ExternalOutcome::Succeeded,
            ExternalActionStage::FailedConfirmed => ExternalOutcome::FailedConfirmed,
            ExternalActionStage::DispatchStarted
            | ExternalActionStage::OutcomeUnknown
            | ExternalActionStage::Reconciling
            | ExternalActionStage::Unresolved => ExternalOutcome::OutcomeUnknown,
        }
    }
}

/// The watchdog's protective dispatch fact: what was admitted at the D boundary
/// before OS handoff (`action_dispatch_started`). It carries no outcome by
/// itself, so a missing observation leaves the outcome unknown — even if the
/// failure might have happened before the first byte (contract 14 §5.3).
///
/// The receipt pins the dispatch-time authority identity (installation, run
/// generation, epoch) as *historical evidence of where the dispatch happened*,
/// and the complete support-fence set of the admitting lease — every fence the
/// admission relied on (root work, unit, lease, parent lease), not one
/// arbitrary fence. Both facts exist so a later reconciliation can say exactly
/// which dispatch it settles; neither is re-required to be *current* for a
/// protective reconciliation to proceed (contract 18 §3.5: recovery starts a
/// new run generation precisely so old unknowns can be reconciled under it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchReceipt {
    pub intent_id: EntityId,
    pub installation_id: EntityId,
    pub run_generation: RunGeneration,
    pub authority_epoch: AuthorityEpoch,
    /// Complete fence support-set captured at dispatch: the exact scopes and
    /// versions the admission depended on. Never a subset of one.
    pub support_fences: BTreeMap<FenceScope, RowVersion>,
}

/// One dispatched intent plus its current evidence stage and outcome evidence.
/// Durable storage and the `authority.anchor` row are integration work; this is the value snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchRecord {
    pub receipt: DispatchReceipt,
    pub stage: ExternalActionStage,
    /// Persisted outcome evidence (observation ID + kind + timestamp)
    /// Required by P1-OUTCOME-EVIDENCE-DROPPED: never reduce "succeeded" to enum without proof reference.
    pub outcome_evidence: Option<OutcomeEvidence>,
}

/// Evidence of an external action outcome with full provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeEvidence {
    /// The observation that produced this outcome
    pub observation_id: EntityId,
    /// Kind of observation (direct vs reconciled)
    pub observation_kind: ObservationKind,
    /// When the observation was recorded
    pub observed_at_utc_millis: u64,
    /// Evidence ID from the observation
    pub evidence_id: EntityId,
    /// Optional reconciliation ID if this came from recovery
    pub reconciliation_id: Option<EntityId>,
}

/// How the outcome was observed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    /// Direct observation from the broker/dispatcher
    Direct,
    /// Recovered through reconciliation after restart
    Reconciled,
}

/// Record the dispatch fact. The new record starts at `DispatchStarted`: it
/// admits no outcome and therefore admits uncertainty on a missing observation.
///
/// Fail-closed identity check (P2-OUTCOME-ID-ALIAS): `intent_id` and
/// `installation_id` are different identity classes and must never alias. The
/// global C-03 store enforces global uniqueness; this boundary rejects the
/// aliased receipt earlier, at the exact point a dispatch enters the watchdog.
pub fn record_dispatch(receipt: DispatchReceipt) -> Result<DispatchRecord, OutcomeError> {
    if receipt.intent_id == receipt.installation_id {
        return Err(OutcomeError::IdentityAlias);
    }
    Ok(DispatchRecord {
        receipt,
        stage: ExternalActionStage::DispatchStarted,
        outcome_evidence: None,
    })
}

/// Record an outcome observation with full evidence preservation.
pub fn record_outcome(
    record: &mut DispatchRecord,
    observation: ActionObservation,
    observation_id: EntityId,
    observed_at_utc_millis: u64,
    reconciliation_id: Option<EntityId>,
) -> Result<(), OutcomeError> {
    match observation {
        ActionObservation::Succeeded { evidence_id } => {
            record.stage = ExternalActionStage::Succeeded;
            record.outcome_evidence = Some(OutcomeEvidence {
                observation_id,
                observation_kind: if reconciliation_id.is_some() {
                    ObservationKind::Reconciled
                } else {
                    ObservationKind::Direct
                },
                observed_at_utc_millis,
                evidence_id,
                reconciliation_id,
            });
            Ok(())
        }
        ActionObservation::Failed { evidence_id } => {
            record.stage = ExternalActionStage::FailedConfirmed;
            record.outcome_evidence = Some(OutcomeEvidence {
                observation_id,
                observation_kind: if reconciliation_id.is_some() {
                    ObservationKind::Reconciled
                } else {
                    ObservationKind::Direct
                },
                observed_at_utc_millis,
                evidence_id,
                reconciliation_id,
            });
            Ok(())
        }
        ActionObservation::Unconfirmed => {
            // Unconfirmed observations do not advance the stage
            // but we still record the attempt for audit
            Ok(())
        }
    }
}

/// A broker-supplied observation of a dispatched action's external outcome.
/// Only a confirmed success or a confirmed failure advances the evidence; an
/// unconfirmed observation proves nothing about the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionObservation {
    Succeeded { evidence_id: EntityId },
    Failed { evidence_id: EntityId },
    Unconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutcomeError {
    #[error("late outcome for an already settled result is rejected")]
    LateResult,
    #[error(
        "an unknown outcome requires an explicit reconciliation before any further observation"
    )]
    RequiresReconciliation,
    #[error("intent_id and installation_id must be distinct identities (P2-OUTCOME-ID-ALIAS)")]
    IdentityAlias,
}

/// Apply an observation to a dispatched intent's evidence stage.
///
/// * `DispatchStarted` + confirmed observation settles success/failure; a
///   confirmless observation classifies as `OutcomeUnknown`.
/// * `Reconciling` + confirmed observation settles success/failure; a
///   confirmless observation becomes `Unresolved` (never a fabricated failure).
/// * `OutcomeUnknown` refuses a plain re-observation: the only forward path is
///   `begin_reconciliation` (contract 14 §7.4 forbids returning to `authorized`).
/// * Any already-terminal stage (`Succeeded`, `FailedConfirmed`, `Unresolved`)
///   rejects a late result (contract 21 AC-L04B).
pub fn observe_outcome(
    record: &DispatchRecord,
    observation: ActionObservation,
) -> Result<DispatchRecord, OutcomeError> {
    let next = match (record.stage, observation) {
        (ExternalActionStage::DispatchStarted, ActionObservation::Succeeded { .. }) => {
            ExternalActionStage::Succeeded
        }
        (ExternalActionStage::DispatchStarted, ActionObservation::Failed { .. }) => {
            ExternalActionStage::FailedConfirmed
        }
        (ExternalActionStage::DispatchStarted, ActionObservation::Unconfirmed) => {
            ExternalActionStage::OutcomeUnknown
        }
        (ExternalActionStage::Reconciling, ActionObservation::Succeeded { .. }) => {
            ExternalActionStage::Succeeded
        }
        (ExternalActionStage::Reconciling, ActionObservation::Failed { .. }) => {
            ExternalActionStage::FailedConfirmed
        }
        (ExternalActionStage::Reconciling, ActionObservation::Unconfirmed) => {
            ExternalActionStage::Unresolved
        }
        (ExternalActionStage::OutcomeUnknown, _) => {
            return Err(OutcomeError::RequiresReconciliation);
        }
        (
            ExternalActionStage::Succeeded
            | ExternalActionStage::FailedConfirmed
            | ExternalActionStage::Unresolved,
            _,
        ) => return Err(OutcomeError::LateResult),
    };
    Ok(DispatchRecord {
        receipt: record.receipt.clone(),
        stage: next,
        outcome_evidence: None,
    })
}

/// A typed, permitted reconciliation authority for one external intent. It is
/// issued by the trusted boundary (C-05/C-09 through C-09's `reconcile_intent`)
/// under the *current* run generation and epoch, and never constructed by a
/// model; this module only re-checks it against the dispatch snapshot and the
/// current authority view. The grant pins the exact intent it may settle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconciliationGrant {
    pub intent_id: EntityId,
    pub action_kind: ActionKind,
    /// Generation the grant was issued under. Must equal the current
    /// authority's, never the historical dispatch's: reconciliation after a
    /// crash is a new-generation protective act (contract 18 §3.5).
    pub run_generation: RunGeneration,
    pub authority_epoch: AuthorityEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReconciliationError {
    #[error("the intent is not in an unknown-outcome state")]
    NotUnknown,
    #[error("the reconciliation grant references a different intent")]
    IntentMismatch,
    #[error("the reconciliation grant does not carry the ReconcileExternalIntent action kind")]
    WrongActionKind,
    #[error("the authority installation differs from the dispatch receipt")]
    WrongInstallation,
    #[error("the reconciliation grant does not match the current authority generation/epoch")]
    StaleGrant,
    #[error("the dispatch-time support fence set is absent from the current authority view")]
    FenceHistoryLost,
}

/// Start a permitted reconciliation of an unknown external outcome.
///
/// Reconciliation is a protective evidence act, not a new dispatch: it must
/// survive exactly the events that close new dispatches — a crash (new run
/// generation), a revoke, a scope stop, a fence bump — because finding out
/// whether an already-sent effect happened is how the system avoids both a
/// blind repeat and a fabricated failure (contract 14 §7.4, 18 §3.5).
///
/// Checks, all fail-closed:
///
/// * the record is in the unknown-outcome state;
/// * the grant names this exact intent and carries `ReconcileExternalIntent`;
/// * the grant was issued under the *current* installation, run generation and
///   epoch — a grant replayed from before a crash or a global epoch bump is
///   stale and refuses;
/// * every dispatch-time support fence is still *known* to the authority view
///   (its history is not lost); closure or version change of those fences is
///   expected after a revoke/stop and does NOT block reconciliation.
///
/// A successful call only moves the record to `Reconciling`; it never re-admits
/// the intent for dispatch, never re-opens a closed fence and never revives a
/// revoked lease. Proving the result still requires `observe_outcome` with a
/// confirmed reconciliation observation (contract 18 §3.5: `action_reconciled`
/// requires a confirmed result and proof).
pub fn begin_reconciliation(
    record: &DispatchRecord,
    grant: &ReconciliationGrant,
    authority: &AuthoritySnapshot,
) -> Result<DispatchRecord, ReconciliationError> {
    if record.stage != ExternalActionStage::OutcomeUnknown {
        return Err(ReconciliationError::NotUnknown);
    }
    let receipt = &record.receipt;
    if grant.intent_id != receipt.intent_id {
        return Err(ReconciliationError::IntentMismatch);
    }
    if grant.action_kind != ActionKind::ReconcileExternalIntent {
        return Err(ReconciliationError::WrongActionKind);
    }
    // The dispatch receipt is historical evidence: the reconciliation must be
    // issued by the same installation, but under the current generation/epoch.
    if authority.installation_id != receipt.installation_id {
        return Err(ReconciliationError::WrongInstallation);
    }
    if grant.run_generation != authority.run_generation
        || grant.authority_epoch != authority.authority_epoch
    {
        return Err(ReconciliationError::StaleGrant);
    }
    // The authority view must still know every fence the dispatch relied on.
    // A closed or re-versioned fence is the normal consequence of a revoke or
    // a stop and never blocks reconciliation; a *forgotten* fence means the
    // authority history is incomplete and nothing may be settled on it.
    for scope in receipt.support_fences.keys() {
        if !authority.fences.contains_key(scope) {
            return Err(ReconciliationError::FenceHistoryLost);
        }
    }
    Ok(DispatchRecord {
        receipt: receipt.clone(),
        stage: ExternalActionStage::Reconciling,
        outcome_evidence: None,
    })
}

/// The contract 18 §9.1 step 10 partition of a recovery batch into confirmed
/// success, confirmed failure and everything still unknown. `unknown` keeps any
/// unproven outcome visible; it is what must be reconciled (or refused) before
/// a resume can be prepared.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryOutcomes {
    pub succeeded: Vec<EntityId>,
    pub failed_confirmed: Vec<EntityId>,
    pub unknown: Vec<EntityId>,
}

impl RecoveryOutcomes {
    /// Fail closed: any unproven outcome blocks an automatic continuation.
    pub fn has_unknown(&self) -> bool {
        !self.unknown.is_empty()
    }
}

/// Classify a recovery batch into its three buckets. A dispatch without an
/// observation, a pending reconciliation and an unresolved reconciliation all
/// land in `unknown`, which surfaces the held cost instead of silently dropping
/// it (contract 18 §9.1 step 10, §8 step 8).
pub fn classify_recovery_outcomes<'a>(
    records: impl IntoIterator<Item = &'a DispatchRecord>,
) -> RecoveryOutcomes {
    let mut classified = RecoveryOutcomes::default();
    for record in records {
        match record.stage.outcome() {
            ExternalOutcome::Succeeded => classified.succeeded.push(record.receipt.intent_id),
            ExternalOutcome::FailedConfirmed => {
                classified.failed_confirmed.push(record.receipt.intent_id)
            }
            ExternalOutcome::OutcomeUnknown => classified.unknown.push(record.receipt.intent_id),
        }
    }
    classified
}
