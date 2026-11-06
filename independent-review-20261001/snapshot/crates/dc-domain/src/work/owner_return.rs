//! C-01 return of a reviewed completion for repair within the current objective.
//! Trusted owner events and accepted assessments are caller-supplied, not
//! authenticated here. This is one proposed atomic change, never a run permit.

use super::lifecycle::{check_root_basis, next_row};
use super::*;

/// Native C-05 must resolve the authenticated owner event and retain its reason.
/// This is not a wire input or proof that a model speaks for the owner.
/// ```compile_fail
/// use dc_domain::work::OwnerRejectionDecision;
/// let _: OwnerRejectionDecision = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerRejectionDecision {
    pub owner_event_id: EntityId,
    pub candidate_id: EntityId,
    pub scope: WorkScope,
    pub final_verification_id: EntityId,
    pub reason_evidence_id: EntityId,
}

/// Accepted assessment, not raw model output. C-08 must establish the evidence
/// and that the repair really lies within the existing objective. C-01 checks
/// exact bindings and versions, not the semantic truth of an evidence reference.
/// ```compile_fail
/// use dc_domain::work::OwnerReturnReview;
/// let _: OwnerReturnReview = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerReturnReview {
    pub assessment_id: EntityId,
    pub owner_event_id: EntityId,
    pub reason_evidence_id: EntityId,
    pub candidate_id: EntityId,
    pub scope: WorkScope,
    pub final_verification_id: EntityId,
    pub assessment_evidence_id: EntityId,
    pub expected_graph_version: RowVersion,
    pub expected_objective_row_version: RowVersion,
    pub repair_obligation_ids: Vec<EntityId>,
    pub publication_assessments: Vec<PublicationImpactAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationImpactAssessment {
    pub publication_id: EntityId,
    pub expected_row_version: RowVersion,
    pub effect: PublicationImpact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationImpact {
    Affected,
    Unknown,
    /// Reference to trusted machine evidence, never an unauthenticated flag.
    ProvenUnaffected {
        evidence_id: EntityId,
    },
}

/// Every member must be committed together. In particular, applying only the
/// root replacement would leave dependent work using unresolved publications.
/// ```compile_fail
/// use dc_domain::work::OwnerReturnPlan;
/// let _: OwnerReturnPlan = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerReturnPlan {
    replacement: RootLifecycleState,
    expected_root_row: RowVersion,
    expected_graph_row: RowVersion,
    expected_objective_row: RowVersion,
    publication_reads: Vec<PublicationState>,
    publication_replacements: Vec<PublicationState>,
    resolved_rejection: PendingOwnerRejection,
    review: OwnerReturnReview,
}

impl OwnerReturnPlan {
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
    /// Includes unchanged and already-ineligible states, not just writes.
    pub fn publication_reads(&self) -> &[PublicationState] {
        &self.publication_reads
    }
    pub fn publication_replacements(&self) -> &[PublicationState] {
        &self.publication_replacements
    }
    pub fn returned_completion(&self) -> &ReviewedCompletion {
        self.resolved_rejection.completion()
    }
    pub fn decision(&self) -> &OwnerRejectionDecision {
        self.resolved_rejection.decision()
    }
    pub fn resolved_rejection(&self) -> &PendingOwnerRejection {
        &self.resolved_rejection
    }
    pub fn review(&self) -> &OwnerReturnReview {
        &self.review
    }
}

/// Records an accepted rejection assessment, not its analysis or execution.
/// Rejection is valid even when old completion evidence has become stale.
/// C-03 must recheck the root, graph, objective and ALL publication read rows
/// atomically, retain this plan's provenance, and prevent global ID reuse.
/// The complete protected graph and genuine evidence remain caller obligations.
pub fn plan_owner_return(
    state: &RootLifecycleState,
    expected_row_version: RowVersion,
    current_scope: WorkScope,
    graph: &ValidatedGraph<'_>,
    decision: &OwnerRejectionDecision,
    review: &OwnerReturnReview,
) -> Result<OwnerReturnPlan, WorkError> {
    check_root_basis(state, expected_row_version, current_scope)?;
    if state.root.stage != RootStage::AwaitingOwnerAcceptance {
        return Err(WorkError::InvalidTransition);
    }
    let pending = state
        .pending_owner_rejection
        .as_ref()
        .ok_or(WorkError::OwnerRejectionMismatch)?;
    if pending.decision() != decision {
        return Err(WorkError::OwnerRejectionMismatch);
    }
    let final_review = state
        .final_review
        .as_ref()
        .ok_or(WorkError::OwnerRejectionMismatch)?;
    if decision.candidate_id != final_review.candidate().candidate_id()
        || decision.scope != current_scope
        || decision.final_verification_id != final_review.verification_id()
        || review.candidate_id != decision.candidate_id
        || review.scope != current_scope
        || review.final_verification_id != decision.final_verification_id
        || review.owner_event_id != decision.owner_event_id
        || review.reason_evidence_id != decision.reason_evidence_id
    {
        return Err(WorkError::OwnerRejectionMismatch);
    }
    if review.repair_obligation_ids.is_empty() || review.repair_obligation_ids.len() > MAX_ITEMS {
        return Err(WorkError::InvalidField("repair_obligation_ids"));
    }
    unique_ids(review.repair_obligation_ids.iter().copied())?;
    if review.publication_assessments.len() > MAX_ITEMS {
        return Err(WorkError::InvalidField("publication_assessments"));
    }

    let g = graph.candidate();
    if g.expected_graph_version != g.current_graph_version
        || review.expected_graph_version != g.current_graph_version
    {
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
    if objective.row_version != review.expected_objective_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    for obligation_id in &review.repair_obligation_ids {
        if !objective
            .obligations
            .iter()
            .any(|o| o.obligation_id == *obligation_id)
        {
            return Err(WorkError::UnknownObligation(*obligation_id));
        }
    }

    let publications: BTreeSet<_> = g
        .publications
        .iter()
        .filter(|p| {
            p.producer_work_id == current_scope.work_id
                && p.producer_objective_id == current_scope.objective_id
        })
        .map(|p| p.publication_id)
        .collect();
    let states: BTreeMap<_, _> = g
        .publication_states
        .iter()
        .map(|s| (s.publication_id, s))
        .collect();
    let mut effects = BTreeMap::new();
    for assessment in &review.publication_assessments {
        if !publications.contains(&assessment.publication_id) {
            return Err(WorkError::UnknownPublication(assessment.publication_id));
        }
        let current = states
            .get(&assessment.publication_id)
            .ok_or(WorkError::UnknownPublication(assessment.publication_id))?;
        if current.row_version != assessment.expected_row_version {
            return Err(WorkError::RowVersionConflict);
        }
        if effects
            .insert(assessment.publication_id, &assessment.effect)
            .is_some()
        {
            return Err(WorkError::Duplicate(assessment.publication_id));
        }
    }
    let mut publication_reads = Vec::with_capacity(publications.len());
    let mut publication_replacements = Vec::new();
    for publication_id in publications {
        let current = *states
            .get(&publication_id)
            .ok_or(WorkError::UnknownPublication(publication_id))?;
        publication_reads.push(current.clone());
        let unaffected = matches!(
            effects.get(&publication_id),
            Some(PublicationImpact::ProvenUnaffected { .. })
        );
        // Only the exact hold installed by this rejection may be lifted.
        // A changed row, another reason or an earlier ineligibility is not ours.
        let owned_hold = pending
            .publication_holds()
            .iter()
            .find(|hold| hold.held() == current);
        if unaffected && let Some(hold) = owned_hold {
            publication_replacements.push(PublicationState {
                row_version: next_row(current.row_version)?,
                ..hold.before().clone()
            });
        } else if current.eligibility == PublicationEligibility::Valid && !unaffected {
            publication_replacements.push(PublicationState {
                publication_id,
                eligibility: PublicationEligibility::Unresolved,
                reason_evidence_id: Some(decision.reason_evidence_id),
                row_version: next_row(current.row_version)?,
            });
        }
    }
    let mut replacement = state.clone();
    replacement.root.stage = RootStage::Executing;
    replacement.root.row_version = next_row(state.root.row_version)?;
    replacement.candidate = None;
    replacement.final_review = None;
    replacement.pending_owner_rejection = None;
    replacement
        .root
        .blocking_reasons
        .remove(&BlockingReason::OwnerRejectionPending {
            owner_event_id: decision.owner_event_id,
        });
    // Only the recorded rejection's own gate is resolved; no other gate,
    // obligation, historical result, lease or resource is reset.
    Ok(OwnerReturnPlan {
        replacement,
        expected_root_row: expected_row_version,
        expected_graph_row: g.current_graph_version,
        expected_objective_row: objective.row_version,
        publication_reads,
        publication_replacements,
        resolved_rejection: pending.clone(),
        review: review.clone(),
    })
}
