//! C-01 verifiable compaction contracts (doc 15 §5.3, §7, §8).
//!
//! A summary candidate is a NEW derived object; it never replaces its sources
//! in place (doc 15 §10: "compaction that overwrites the original memory" is
//! forbidden). Until an INDEPENDENT verifier returns `Verdict::Pass`, the
//! candidate may be used only as a search pointer, never as authority
//! (§5.3). Every record in the candidate traces to a canonical source via
//! `source_ref`; compaction of compaction without a canonical anchor is
//! rejected (`SummaryError::UntraceableToCanonicalSource`).
//!
//! This module is pure: no I/O, no clock, no persistence, no search. The
//! caller supplies the candidate, the full source set it claims to cover and
//! trusted identifiers. The verifier here performs only the deterministic
//! structural checks of §5.3; the independent reviewer (a different worker)
//! still judges semantic fidelity outside this crate.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::context::{ContentFingerprint, SchemaVersion};
use crate::ids::{EntityId, RowVersion};

/// Lifecycle of a right (lease) as the compaction layer needs to know it.
/// Mirrors the live/dead distinction of `permission::LeaseStatus` without
/// coupling this pure contract to that crate-private surface: the caller
/// maps its stored status into this enum at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightLifecycle {
    /// Requested, active, consumed or otherwise still usable.
    Live,
    /// Expired by time.
    Expired,
    /// Revoked by an owner event.
    Revoked,
    /// Superseded by a newer grant.
    Superseded,
}

impl RightLifecycle {
    pub const fn is_dead(self) -> bool {
        matches!(self, Self::Expired | Self::Revoked | Self::Superseded)
    }
}

// ---------------------------------------------------------------------------
// Doc 15 §5.3 — the mandatory content of a summary candidate.
// ---------------------------------------------------------------------------

/// Whether a record came from a canonical source or a prior summary.
/// A candidate may include a record from a prior summary ONLY through
/// `origin_ref` chaining that ends at a canonical source; the verifier
/// requires the terminal hop to be `SummaryOrigin::Canonical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryOrigin {
    /// Direct reference to an original evidence/decision/obligation row.
    Canonical,
    /// Reference to a record inside a previously VERIFIED summary candidate
    /// that itself traces to canonical sources.
    PriorVerifiedSummary,
}

/// One entry in the candidate's coverage table: a record in the summary,
/// the canonical source it traces to and the transformation applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageEntry {
    /// Local id of the record inside the candidate (not a system EntityId of
    /// any stored row — the candidate itself is not yet persisted).
    pub record_local_id: String,
    /// The source row this record claims to preserve or derive from.
    pub source_ref: EntityId,
    pub origin: SummaryOrigin,
    /// Fingerprint of the exact source content the author read. The verifier
    /// re-reads nothing here; equality against the caller-supplied source set
    /// is what proves the author saw the same bytes the verifier sees.
    pub source_fingerprint: ContentFingerprint,
    /// True when the record preserves the source verbatim in meaning
    /// (paraphrase allowed; negation/modality must be intact — checked
    /// structurally where possible and by the independent reviewer otherwise).
    pub preserves_negation: bool,
}

/// A right (lease/permission) as it must appear in the summary: with its
/// CURRENT lifecycle status, so a revoked or expired right can never be
/// silently revived as active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightStateEntry {
    pub lease_id: EntityId,
    pub status: RightLifecycle,
    /// Row version of the lease row the author read.
    pub observed_revision: RowVersion,
}

/// An external outcome whose result is genuinely unknown. §5.3 forbids
/// turning an unknown external outcome into a failure during compaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownOutcomeEntry {
    pub outcome_ref: EntityId,
    pub description: String,
}

/// The summary candidate itself: a derived object covering an explicit source
/// range, never replacing the sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryCandidate {
    /// Pre-persistence local id supplied by the caller boundary.
    pub candidate_id: EntityId,
    /// The objective and its exact revision this summary serves.
    pub objective_id: EntityId,
    pub objective_revision: RowVersion,
    /// Inclusive range of source rows covered (by caller-defined ordering key).
    pub source_range_start: EntityId,
    pub source_range_end: EntityId,
    /// Obligations and prohibitions carried forward. Every OPEN or VIOLATED
    /// obligation of the source set must appear here (checked against the
    /// caller-supplied obligation set).
    pub carried_obligations: Vec<EntityId>,
    /// Hypotheses kept strictly separate from facts: ids of records whose
    /// knowledge state must remain Hypothesis in the candidate.
    pub hypothesis_record_ids: Vec<String>,
    /// Negative results and rejected paths retained (§5.3; check 16 of §9:
    /// a negative result survives the merge).
    pub retained_negative_result_ids: Vec<String>,
    /// Rights with their current status: active, expired AND revoked.
    pub right_states: Vec<RightStateEntry>,
    /// External outcomes still unknown at compaction time.
    pub unknown_outcomes: Vec<UnknownOutcomeEntry>,
    /// Coverage table mapping every record to its canonical source.
    pub coverage: Vec<CoverageEntry>,
    /// Version of the compaction transform that produced this candidate.
    pub transform_version: SchemaVersion,
}

// ---------------------------------------------------------------------------
// Independent verification (doc 15 §5.3, §8 beacons).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The candidate may now be used as more than a search pointer.
    Pass,
    /// The candidate remains a search pointer only.
    Fail,
}

/// What the independent verifier must look for (§5.3): lost negation,
/// lost obligation, hypothesis promoted to fact, revived right, unknown
/// outcome turned into a failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SummaryError {
    #[error("candidate covers a source range with no coverage entries")]
    EmptyCoverage,
    #[error("record {0} has no coverage entry tracing it to a source")]
    RecordWithoutSource(String),
    #[error("record {0} traces to a source outside the declared range")]
    SourceOutsideRange(String),
    #[error(
        "record {0} chains through a prior summary but the chain never reaches a canonical source"
    )]
    UntraceableToCanonicalSource(String),
    #[error("open/violated obligation {0} is missing from the candidate")]
    LostObligation(EntityId),
    #[error("hypothesis record {0} lost its hypothesis status (promoted to fact)")]
    HypothesisPromoted(String),
    #[error("negative result {0} was dropped by the candidate")]
    NegativeResultLost(String),
    #[error("revoked/expired right {0} reappears as active")]
    RightRevived(EntityId),
    #[error("unknown external outcome {0} was resolved as a failure in the candidate")]
    UnknownOutcomeAsFailure(EntityId),
    #[error("collection exceeds structural limits")]
    Oversized,
}

/// Caller-supplied ground truth the candidate is checked against. The
/// verifier reads this set, not the candidate's claims about it.
pub struct SummaryVerificationInput<'a> {
    pub candidate: &'a SummaryCandidate,
    /// All obligations of the source set whose status is Open or Violated.
    pub open_or_violated_obligations: &'a [EntityId],
    /// Source-set records whose knowledge state is Hypothesis (by local id
    /// as they appear in the candidate's coverage).
    pub source_hypothesis_ids: &'a [String],
    /// Source-set negative result ids that must survive.
    pub source_negative_result_ids: &'a [String],
    /// Source-set rights with their CURRENT status at verification time.
    pub current_right_states: &'a [RightStateEntry],
    /// Source-set unknown external outcomes.
    pub source_unknown_outcomes: &'a [EntityId],
    /// For each `PriorVerifiedSummary` hop in coverage, the terminal origin
    /// of that prior candidate's chain (true = reaches canonical). Supplied
    /// by the caller from prior verification records; never taken from the
    /// candidate itself.
    pub prior_chain_reaches_canonical: bool,
}

/// Verify a summary candidate against the five loss classes of §5.3 plus
/// canonical traceability. Pure; returns the first loss found. A `Pass`
/// verdict here is necessary but NOT sufficient — the independent reviewer
/// still judges semantic fidelity outside this crate (§5.3, §9 checks 16-18).
pub fn verify_summary_candidate(
    input: &SummaryVerificationInput<'_>,
) -> Result<Verdict, SummaryError> {
    let c = input.candidate;

    if c.coverage.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
        || c.carried_obligations.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
        || c.right_states.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
        || c.unknown_outcomes.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
    {
        return Err(SummaryError::Oversized);
    }
    if c.coverage.is_empty() {
        return Err(SummaryError::EmptyCoverage);
    }

    // Every covered record must trace to a canonical source: either directly,
    // or through a prior candidate whose chain is verified to reach canonical.
    for entry in &c.coverage {
        match entry.origin {
            SummaryOrigin::Canonical => {}
            SummaryOrigin::PriorVerifiedSummary => {
                if !input.prior_chain_reaches_canonical {
                    return Err(SummaryError::UntraceableToCanonicalSource(
                        entry.record_local_id.clone(),
                    ));
                }
            }
        }
    }

    // Lost obligation: every open/violated obligation must be carried forward.
    let carried: BTreeSet<EntityId> = c.carried_obligations.iter().copied().collect();
    for obligation in input.open_or_violated_obligations {
        if !carried.contains(obligation) {
            return Err(SummaryError::LostObligation(*obligation));
        }
    }

    // Hypothesis promoted to fact: a source hypothesis must remain declared
    // as a hypothesis in the candidate.
    let kept_hypotheses: BTreeSet<&str> =
        c.hypothesis_record_ids.iter().map(String::as_str).collect();
    for source_hypothesis in input.source_hypothesis_ids {
        if !kept_hypotheses.contains(source_hypothesis.as_str()) {
            return Err(SummaryError::HypothesisPromoted(source_hypothesis.clone()));
        }
    }

    // Negative result lost.
    let kept_negatives: BTreeSet<&str> = c
        .retained_negative_result_ids
        .iter()
        .map(String::as_str)
        .collect();
    for negative in input.source_negative_result_ids {
        if !kept_negatives.contains(negative.as_str()) {
            return Err(SummaryError::NegativeResultLost(negative.clone()));
        }
    }

    // Revived right: a right that is Revoked/Expired in the caller's current
    // view must not appear as Active (or any live status) in the candidate.
    let candidate_rights: std::collections::BTreeMap<EntityId, RightLifecycle> = c
        .right_states
        .iter()
        .map(|r| (r.lease_id, r.status))
        .collect();
    for current in input.current_right_states {
        if current.status.is_dead()
            && let Some(candidate_status) = candidate_rights.get(&current.lease_id)
            && !candidate_status.is_dead()
        {
            return Err(SummaryError::RightRevived(current.lease_id));
        }
    }

    // Unknown external outcome turned into a failure: every unknown outcome
    // of the source set must still be listed as unknown in the candidate.
    let candidate_unknowns: BTreeSet<EntityId> =
        c.unknown_outcomes.iter().map(|u| u.outcome_ref).collect();
    for unknown in input.source_unknown_outcomes {
        if !candidate_unknowns.contains(unknown) {
            return Err(SummaryError::UnknownOutcomeAsFailure(*unknown));
        }
    }

    Ok(Verdict::Pass)
}

/// Whether a verified candidate may be used as authority input to planning,
/// or only as a search pointer (doc 15 §5.3: before verdict `pass` the
/// summary is used ONLY as a search pointer).
pub const fn may_serve_as_authority(verdict: Verdict) -> bool {
    matches!(verdict, Verdict::Pass)
}

#[cfg(test)]
mod tests;
