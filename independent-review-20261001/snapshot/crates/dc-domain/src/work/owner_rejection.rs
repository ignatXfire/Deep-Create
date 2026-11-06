//! C-01 recording of a pending owner rejection before its reason is assessed.
//! Trusted inputs, no authentication, I/O, commit receipt or execution authority.
//! Initial impact is unknown. Existing evidence is retained, not withdrawn.

use super::lifecycle::{check_root_basis, next_row};
use super::*;

/// An exact owner decision and the temporary publication holds it installed.
/// ```compile_fail
/// use dc_domain::work::PendingOwnerRejection;
/// let _: PendingOwnerRejection = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOwnerRejection {
    decision: OwnerRejectionDecision,
    completion: ReviewedCompletion,
    publication_holds: Vec<PublicationRejectionHold>,
}

impl PendingOwnerRejection {
    pub fn decision(&self) -> &OwnerRejectionDecision {
        &self.decision
    }
    pub fn completion(&self) -> &ReviewedCompletion {
        &self.completion
    }
    pub fn publication_holds(&self) -> &[PublicationRejectionHold] {
        &self.publication_holds
    }
}

/// Only an exact, unchanged held row may later be restored after trusted proof
/// of non-impact. A prior or concurrent other-cause state is never owned here.
/// ```compile_fail
/// use dc_domain::work::PublicationRejectionHold;
/// let _: PublicationRejectionHold = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationRejectionHold {
    before: PublicationState,
    held: PublicationState,
}

impl PublicationRejectionHold {
    pub fn before(&self) -> &PublicationState {
        &self.before
    }
    pub fn held(&self) -> &PublicationState {
        &self.held
    }
}

/// One atomic proposal: C-03 must recheck the root, graph, objective and ALL
/// publication reads, retain the pending record and apply all replacements.
/// Command deduplication and global never-reused IDs are still C-03 obligations.
/// ```compile_fail
/// use dc_domain::work::OwnerRejectionPlan;
/// let _: OwnerRejectionPlan = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerRejectionPlan {
    replacement: RootLifecycleState,
    expected_root_row: RowVersion,
    expected_graph_row: RowVersion,
    expected_objective_row: RowVersion,
    publication_reads: Vec<PublicationState>,
    publication_replacements: Vec<PublicationState>,
}

impl OwnerRejectionPlan {
    pub fn replacement(&self) -> &RootLifecycleState {
        &self.replacement
    }
    pub fn expected_root_row(&self) -> RowVersion {
        self.expected_root_row
    }
    pub fn expected_graph_row(&self) -> RowVersion {
        self.expected_graph_row
    }
    pub fn expected_objective_row(&self) -> RowVersion {
        self.expected_objective_row
    }
    pub fn publication_reads(&self) -> &[PublicationState] {
        &self.publication_reads
    }
    pub fn publication_replacements(&self) -> &[PublicationState] {
        &self.publication_replacements
    }
}

/// Receive a rejection without requiring an accepted analysis or freshness of
/// the rejected evidence. Candidate/review and semantic stage stay unchanged;
/// the exact new gate blocks acceptance and execution, not recording the fact.
/// An existing pending rejection cannot be overwritten, even by a duplicate.
pub fn plan_owner_rejection(
    state: &RootLifecycleState,
    expected_row_version: RowVersion,
    current_scope: WorkScope,
    graph: &ValidatedGraph<'_>,
    decision: &OwnerRejectionDecision,
) -> Result<OwnerRejectionPlan, WorkError> {
    check_root_basis(state, expected_row_version, current_scope)?;
    if state.root.stage != RootStage::AwaitingOwnerAcceptance
        || state.pending_owner_rejection.is_some()
        || state
            .root
            .blocking_reasons
            .iter()
            .any(|r| matches!(r, BlockingReason::OwnerRejectionPending { .. }))
    {
        return Err(WorkError::InvalidTransition);
    }
    let completion = state
        .final_review
        .as_ref()
        .ok_or(WorkError::ResultBasisMismatch)?;
    if decision.candidate_id != completion.candidate().candidate_id()
        || decision.scope != current_scope
        || decision.final_verification_id != completion.verification_id()
    {
        return Err(WorkError::OwnerRejectionMismatch);
    }
    let g = graph.candidate();
    if g.expected_graph_version != g.current_graph_version {
        return Err(WorkError::GraphVersionConflict);
    }
    if !g.roots.iter().any(|root| root == &state.root) {
        return Err(WorkError::ResultBasisMismatch);
    }
    let objective = g
        .objectives
        .iter()
        .find(|o| o.scope == current_scope)
        .ok_or(WorkError::UnknownObjective(current_scope.objective_id))?;
    let states: BTreeMap<_, _> = g
        .publication_states
        .iter()
        .map(|s| (s.publication_id, s))
        .collect();
    let mut reads = Vec::new();
    for p in g.publications.iter().filter(|p| {
        p.producer_work_id == current_scope.work_id
            && p.producer_objective_id == current_scope.objective_id
    }) {
        let before = *states
            .get(&p.publication_id)
            .ok_or(WorkError::UnknownPublication(p.publication_id))?;
        reads.push(before.clone());
    }
    reads.sort_by_key(|s| s.publication_id);
    let mut holds = Vec::new();
    let mut replacements = Vec::new();
    for before in reads
        .iter()
        .filter(|s| s.eligibility == PublicationEligibility::Valid)
    {
        let held = PublicationState {
            eligibility: PublicationEligibility::Unresolved,
            reason_evidence_id: Some(decision.reason_evidence_id),
            row_version: next_row(before.row_version)?,
            ..before.clone()
        };
        replacements.push(held.clone());
        holds.push(PublicationRejectionHold {
            before: before.clone(),
            held,
        });
    }
    let mut replacement = state.clone();
    replacement.root.row_version = next_row(state.root.row_version)?;
    replacement
        .root
        .blocking_reasons
        .insert(BlockingReason::OwnerRejectionPending {
            owner_event_id: decision.owner_event_id,
        });
    replacement.pending_owner_rejection = Some(PendingOwnerRejection {
        decision: decision.clone(),
        completion: completion.clone(),
        publication_holds: holds,
    });
    Ok(OwnerRejectionPlan {
        replacement,
        expected_root_row: expected_row_version,
        expected_graph_row: g.current_graph_version,
        expected_objective_row: objective.row_version,
        publication_reads: reads,
        publication_replacements: replacements,
    })
}
