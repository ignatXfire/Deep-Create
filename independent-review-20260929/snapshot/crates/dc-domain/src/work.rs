//! C-01 work contracts. Inputs are proposed or canonical caller-supplied views;
//! validation returns no authority, reservation, persistence receipt, or I/O.
//!
//! The complete affected graph must be read and committed atomically by C-03.
//! This module cannot establish that a caller supplied a complete database view.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::{EntityId, RowVersion, UnitRevision};
use crate::structure::STRUCTURAL_LIMITS_V1;

mod commit;
mod continuation;
mod explanation;
mod goal_change;
mod lifecycle;
mod links;
mod owner_rejection;
mod owner_return;
mod result;
pub use result::*;
pub use commit::*;
pub use continuation::*;
pub use explanation::*;
pub use goal_change::*;
pub use lifecycle::*;
pub use links::*;
pub use owner_rejection::*;
pub use owner_return::*;

pub const MAX_TEXT_BYTES: usize = STRUCTURAL_LIMITS_V1.max_text_bytes;
pub const MAX_ITEMS: usize = STRUCTURAL_LIMITS_V1.max_collection_items;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkScope {
    pub session_id: EntityId,
    pub work_id: EntityId,
    pub objective_id: EntityId,
    pub objective_revision: UnitRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitRef {
    pub unit_id: EntityId,
    pub revision: UnitRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootStage {
    Draft,
    Executing,
    CompletionCandidate,
    Verifying,
    AwaitingOwnerAcceptance,
    OwnerAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitStage {
    Proposed,
    Ready,
    Leased,
    Running,
    ResultSubmitted,
    Verifying,
    Accepted,
    Rejected,
    Stale,
    Blocked,
    Cancelling,
    Cancelled,
    Lost,
}

/// Orthogonal to the semantic stage: clearing one cause cannot clear others.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockingReason {
    Recovery {},
    IntegrityFailure {},
    ResourceUnavailable {},
    StopRequested {},
    UnknownOutcome {},
    AwaitingOwner {},
    OwnerRejectionPending { owner_event_id: EntityId },
    AwaitingPublication { link_id: EntityId },
    PublicationUnresolved { publication_id: EntityId },
    PublicationWithdrawn { publication_id: EntityId },
    OwnerAcceptancePending { publication_id: EntityId },
    ConditionUnknown { link_id: EntityId },
    UpstreamCancelled { unit: UnitRef },
    DependencyPending { unit: UnitRef },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootWork {
    pub work_id: EntityId,
    pub session_id: EntityId,
    pub active_objective_id: EntityId,
    pub stage: RootStage,
    pub blocking_reasons: BTreeSet<BlockingReason>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationKind {
    Required,
    Forbidden,
    Preference,
    Acceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Open,
    Satisfied,
    Violated,
    Waived,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    Deterministic,
    IndependentReview,
    OwnerJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub obligation_id: EntityId,
    pub objective_id: EntityId,
    pub parent_id: Option<EntityId>,
    pub kind: ObligationKind,
    pub statement: String,
    pub verification_method: VerificationMethod,
    pub status: ObligationStatus,
    /// This reference is supplied only from the authenticated owner-event store.
    pub waiver_owner_event_id: Option<EntityId>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveContract {
    pub scope: WorkScope,
    pub statement: String,
    pub owner_event_ids: Vec<EntityId>,
    pub obligations: Vec<Obligation>,
    /// Exact obligation IDs, never a free-form executable success predicate.
    pub completion_criteria: Vec<EntityId>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerConstraints {
    pub required_obligations: Vec<EntityId>,
    pub forbidden_obligations: Vec<EntityId>,
}

#[derive(Debug)]
pub struct ValidatedObjective<'a>(&'a ObjectiveContract);

impl<'a> ValidatedObjective<'a> {
    pub fn contract(&self) -> &'a ObjectiveContract {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkKind {
    Planning,
    Research,
    CreativeExploration,
    Synthesis,
    Summary,
    PreflightAnalysis,
    LocalChangeProposal,
    ModelAdaptation,
    DeterministicCheck,
    Verification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopCondition {
    OutputProduced,
    EvidenceSaturated,
    StepLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkBounds {
    pub max_attempts: NonZeroU32,
    pub max_steps: NonZeroU32,
    pub stop_conditions: Vec<StopCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplorationContract {
    pub question: String,
    pub allowed_scope: String,
    pub max_expansions: NonZeroU32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkUnitSpec {
    pub unit: UnitRef,
    pub scope: WorkScope,
    pub parent: Option<UnitRef>,
    pub kind: WorkKind,
    pub expected_output: OutputContract,
    pub obligation_ids: Vec<EntityId>,
    pub acceptance_criteria: Vec<EntityId>,
    pub bounds: WorkBounds,
    pub exploration: Option<ExplorationContract>,
    pub stage: UnitStage,
    pub accepted_result: Option<AcceptedResultRef>,
    pub row_version: RowVersion,
}

/// Canonical binding populated with an accepted packet by C-03.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedResultRef {
    pub result_id: EntityId,
    pub snapshot_id: EntityId,
    pub fingerprint: [u8; 32],
    pub result_row_version: RowVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRelation {
    MustFinish,
    MustSupply,
    MustResolve,
    MustAuthorize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkDependency {
    pub consumer: UnitRef,
    pub producer: UnitRef,
    pub relation: DependencyRelation,
}

/// Structural limits only. Resource accounting belongs to the resource module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphLimits {
    pub max_units: NonZeroU32,
    pub max_edges: NonZeroU32,
    pub max_attempts_per_unit: NonZeroU32,
    pub max_steps_per_unit: NonZeroU32,
    pub max_exploration_expansions: NonZeroU32,
}

/// Mixed canonical/proposal view, not a wire DTO or a persistent schema.
/// ```compile_fail
/// use dc_domain::work::GraphCandidate;
/// let _: GraphCandidate = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphCandidate {
    pub expected_graph_version: RowVersion,
    pub current_graph_version: RowVersion,
    pub roots: Vec<RootWork>,
    pub objectives: Vec<ObjectiveContract>,
    pub units: Vec<WorkUnitSpec>,
    pub dependencies: Vec<WorkDependency>,
    pub links: Vec<RootWorkLink>,
    pub publications: Vec<ResultPublication>,
    pub publication_states: Vec<PublicationState>,
    pub publication_bindings: Vec<PublicationBinding>,
    pub owner_acceptances: Vec<OwnerAcceptanceBinding>,
}

#[derive(Debug)]
pub struct ValidatedGraph<'a> {
    pub(crate) graph: &'a GraphCandidate,
}

impl<'a> ValidatedGraph<'a> {
    pub fn candidate(&self) -> &'a GraphCandidate {
        self.graph
    }
    pub fn unit(&self, unit: UnitRef) -> Option<&'a WorkUnitSpec> {
        self.graph.units.iter().find(|u| u.unit == unit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkError {
    #[error("missing or invalid criterion")]
    MissingCriterion,
    #[error("owner constraint conflicts with proposed objective")]
    ConstraintConflict,
    #[error("invalid or oversized field: {0}")]
    InvalidField(&'static str),
    #[error("duplicate identity or binding: {0}")]
    Duplicate(EntityId),
    #[error("foreign scope: {0}")]
    ForeignScope(EntityId),
    #[error("unknown unit revision: {0:?}")]
    UnknownUnit(UnitRef),
    #[error("unknown objective: {0}")]
    UnknownObjective(EntityId),
    #[error("unknown obligation: {0}")]
    UnknownObligation(EntityId),
    #[error("unbound obligation: {0}")]
    UnboundObligation(EntityId),
    #[error("unit has no obligation binding: {0:?}")]
    UnboundUnit(UnitRef),
    #[error("unbounded unit: {0:?}")]
    UnboundedUnit(UnitRef),
    #[error("cycle in affected graph")]
    Cycle,
    #[error("affected graph version changed")]
    GraphVersionConflict,
    #[error("unknown publication: {0}")]
    UnknownPublication(EntityId),
    #[error("invalid link condition")]
    InvalidLinkCondition,
    #[error("output schema mismatch")]
    OutputSchemaMismatch,
    #[error("unsupported link policy")]
    UnsupportedLinkPolicy,
    #[error("dependency requires a separate canonical authority predicate: {0:?}")]
    UnsupportedDependencyRelation(DependencyRelation),
    #[error("stale dependency: {0:?}")]
    StaleInput(DependencyKey),
    #[error("incomplete dependency snapshot: {0:?}")]
    IncompleteSnapshot(DependencyKey),
    #[error("invalid result transition")]
    InvalidTransition,
    #[error("result basis does not match exact current unit, objective or snapshot")]
    ResultBasisMismatch,
    #[error("verification does not match exact candidate and policy")]
    VerificationMismatch,
    #[error("object row version changed")]
    RowVersionConflict,
    #[error("version arithmetic overflow")]
    VersionOverflow,
    #[error("owner acceptance does not match the exact final candidate")]
    OwnerAcceptanceMismatch,
    #[error("owner rejection or its assessment does not match the exact candidate")]
    OwnerRejectionMismatch,
    #[error("dependency is currently blocked: {0:?}")]
    Blocked(BlockingReason),
}

pub(crate) fn bounded_text(text: &str, field: &'static str) -> Result<(), WorkError> {
    if text.trim().is_empty() || text.len() > MAX_TEXT_BYTES {
        return Err(WorkError::InvalidField(field));
    }
    Ok(())
}

pub(crate) fn unique_ids(ids: impl IntoIterator<Item = EntityId>) -> Result<(), WorkError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(WorkError::Duplicate(id));
        }
    }
    Ok(())
}

pub fn validate_objective<'a>(
    objective: &'a ObjectiveContract,
    constraints: &OwnerConstraints,
) -> Result<ValidatedObjective<'a>, WorkError> {
    bounded_text(&objective.statement, "objective.statement")?;
    if objective.owner_event_ids.is_empty() || objective.owner_event_ids.len() > MAX_ITEMS {
        return Err(WorkError::InvalidField("owner_event_ids"));
    }
    unique_ids(objective.owner_event_ids.iter().copied())?;
    if objective.obligations.is_empty()
        || objective.obligations.len() > MAX_ITEMS
        || objective.completion_criteria.is_empty()
        || objective.completion_criteria.len() > MAX_ITEMS
    {
        return Err(WorkError::MissingCriterion);
    }
    unique_ids(objective.obligations.iter().map(|o| o.obligation_id))?;
    unique_ids(objective.completion_criteria.iter().copied())?;
    let obligations: BTreeMap<_, _> = objective
        .obligations
        .iter()
        .map(|o| (o.obligation_id, o))
        .collect();
    let mut parent_edges = Vec::new();
    for o in &objective.obligations {
        if o.objective_id != objective.scope.objective_id {
            return Err(WorkError::ForeignScope(o.obligation_id));
        }
        bounded_text(&o.statement, "obligation.statement")?;
        if (o.status == ObligationStatus::Waived) != o.waiver_owner_event_id.is_some() {
            return Err(WorkError::ConstraintConflict);
        }
        if let Some(parent) = o.parent_id {
            if !obligations.contains_key(&parent) {
                return Err(WorkError::UnknownObligation(parent));
            }
            parent_edges.push((o.obligation_id, parent));
        }
    }
    reject_cycle(obligations.keys().copied(), &parent_edges)?;
    for id in &objective.completion_criteria {
        let o = obligations
            .get(id)
            .ok_or(WorkError::UnknownObligation(*id))?;
        if o.kind == ObligationKind::Preference {
            return Err(WorkError::ConstraintConflict);
        }
    }
    if constraints.required_obligations.len() > MAX_ITEMS
        || constraints.forbidden_obligations.len() > MAX_ITEMS
    {
        return Err(WorkError::InvalidField("owner_constraints"));
    }
    for (ids, kind) in [
        (&constraints.required_obligations, ObligationKind::Required),
        (
            &constraints.forbidden_obligations,
            ObligationKind::Forbidden,
        ),
    ] {
        for id in ids {
            if obligations
                .get(id)
                .is_none_or(|o| o.kind != kind || o.status == ObligationStatus::Waived)
            {
                return Err(WorkError::ConstraintConflict);
            }
        }
    }
    Ok(ValidatedObjective(objective))
}

/// Kahn's algorithm avoids recursion on untrusted graphs. Edges are waits-on.
pub(crate) fn reject_cycle<T: Ord + Copy>(
    nodes: impl IntoIterator<Item = T>,
    edges: &[(T, T)],
) -> Result<(), WorkError> {
    let mut degree: BTreeMap<T, usize> = nodes.into_iter().map(|n| (n, 0)).collect();
    let mut outgoing: BTreeMap<T, Vec<T>> = BTreeMap::new();
    for &(from, to) in edges {
        *degree.entry(to).or_default() += 1;
        degree.entry(from).or_default();
        outgoing.entry(from).or_default().push(to);
    }
    let mut ready: BTreeSet<_> = degree
        .iter()
        .filter_map(|(n, d)| (*d == 0).then_some(*n))
        .collect();
    let mut visited = 0;
    while let Some(node) = ready.pop_first() {
        visited += 1;
        if let Some(next) = outgoing.get(&node) {
            for target in next {
                if let Some(value) = degree.get_mut(target) {
                    *value -= 1;
                    if *value == 0 {
                        ready.insert(*target);
                    }
                }
            }
        }
    }
    if visited != degree.len() {
        return Err(WorkError::Cycle);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WaitNode {
    Unit(UnitRef),
    OwnerAcceptance(EntityId),
}

pub fn validate_work_graph<'a>(
    graph: &'a GraphCandidate,
    limits: &GraphLimits,
) -> Result<ValidatedGraph<'a>, WorkError> {
    if graph.expected_graph_version != graph.current_graph_version {
        return Err(WorkError::GraphVersionConflict);
    }
    if graph.roots.is_empty()
        || graph.roots.len() > STRUCTURAL_LIMITS_V1.max_roots
        || graph.objectives.len() > MAX_ITEMS
        || graph.units.len() > MAX_ITEMS
        || graph.units.len() > limits.max_units.get() as usize
        || graph.dependencies.len() > MAX_ITEMS
        || graph.links.len() > MAX_ITEMS
        || graph.publications.len() > MAX_ITEMS
        || graph.publication_states.len() > MAX_ITEMS
        || graph.publication_bindings.len() > MAX_ITEMS
        || graph.owner_acceptances.len() > MAX_ITEMS
    {
        return Err(WorkError::InvalidField("graph.size"));
    }
    unique_ids(graph.roots.iter().map(|r| r.work_id))?;
    unique_ids(graph.objectives.iter().map(|o| o.scope.objective_id))?;
    unique_ids(
        graph
            .objectives
            .iter()
            .flat_map(|o| o.obligations.iter().map(|v| v.obligation_id)),
    )?;
    unique_ids(graph.units.iter().map(|u| u.unit.unit_id))?;
    unique_ids(
        graph
            .units
            .iter()
            .filter_map(|u| u.accepted_result.as_ref().map(|r| r.result_id)),
    )?;
    let roots: BTreeMap<_, _> = graph.roots.iter().map(|r| (r.work_id, r)).collect();
    let objectives: BTreeMap<_, _> = graph
        .objectives
        .iter()
        .map(|o| (o.scope.objective_id, o))
        .collect();
    let units: BTreeMap<_, _> = graph.units.iter().map(|u| (u.unit, u)).collect();
    for root in &graph.roots {
        let objective = objectives
            .get(&root.active_objective_id)
            .ok_or(WorkError::UnknownObjective(root.active_objective_id))?;
        if objective.scope.work_id != root.work_id || objective.scope.session_id != root.session_id
        {
            return Err(WorkError::ForeignScope(root.work_id));
        }
    }
    for objective in &graph.objectives {
        validate_objective(objective, &OwnerConstraints::default())?;
        let root = roots
            .get(&objective.scope.work_id)
            .ok_or(WorkError::ForeignScope(objective.scope.work_id))?;
        if root.session_id != objective.scope.session_id {
            return Err(WorkError::ForeignScope(objective.scope.objective_id));
        }
    }
    let mut bound = BTreeSet::new();
    let mut ancestry = Vec::new();
    for unit in &graph.units {
        if (unit.stage == UnitStage::Accepted) != unit.accepted_result.is_some() {
            return Err(WorkError::InvalidField("unit.accepted_result"));
        }
        let objective = objectives
            .get(&unit.scope.objective_id)
            .ok_or(WorkError::UnknownObjective(unit.scope.objective_id))?;
        if unit.scope != objective.scope {
            return Err(WorkError::ForeignScope(unit.unit.unit_id));
        }
        if unit.bounds.max_attempts > limits.max_attempts_per_unit
            || unit.bounds.max_steps > limits.max_steps_per_unit
            || unit.bounds.stop_conditions.is_empty()
            || unit.bounds.stop_conditions.len() > 3
        {
            return Err(WorkError::UnboundedUnit(unit.unit));
        }
        unit.expected_output.validate()?;
        if unit.obligation_ids.is_empty() {
            return Err(WorkError::UnboundUnit(unit.unit));
        }
        if unit.obligation_ids.len() > MAX_ITEMS
            || unit.acceptance_criteria.is_empty()
            || unit.acceptance_criteria.len() > MAX_ITEMS
        {
            return Err(WorkError::MissingCriterion);
        }
        unique_ids(unit.obligation_ids.iter().copied())?;
        unique_ids(unit.acceptance_criteria.iter().copied())?;
        for id in unit.obligation_ids.iter().chain(&unit.acceptance_criteria) {
            if !objective.obligations.iter().any(|o| o.obligation_id == *id) {
                return Err(WorkError::UnknownObligation(*id));
            }
        }
        if !unit
            .acceptance_criteria
            .iter()
            .all(|id| unit.obligation_ids.contains(id))
        {
            return Err(WorkError::ConstraintConflict);
        }
        bound.extend(
            unit.obligation_ids
                .iter()
                .map(|id| (unit.scope.objective_id, *id)),
        );
        if let Some(exploration) = &unit.exploration {
            bounded_text(&exploration.question, "exploration.question")?;
            bounded_text(&exploration.allowed_scope, "exploration.allowed_scope")?;
            if exploration.max_expansions > limits.max_exploration_expansions {
                return Err(WorkError::UnboundedUnit(unit.unit));
            }
        }
        if let Some(parent) = unit.parent {
            let source = units.get(&parent).ok_or(WorkError::UnknownUnit(parent))?;
            if source.scope != unit.scope {
                return Err(WorkError::ForeignScope(parent.unit_id));
            }
            ancestry.push((unit.unit, parent));
        }
    }
    reject_cycle(units.keys().copied(), &ancestry)?;
    for objective in &graph.objectives {
        for obligation in &objective.obligations {
            if obligation.kind != ObligationKind::Preference
                && obligation.status != ObligationStatus::Waived
                && obligation.verification_method != VerificationMethod::OwnerJudgment
                && !bound.contains(&(objective.scope.objective_id, obligation.obligation_id))
            {
                return Err(WorkError::UnboundObligation(obligation.obligation_id));
            }
        }
    }
    let mut waits = Vec::new();
    let mut pairs = BTreeSet::new();
    for dependency in &graph.dependencies {
        let consumer = units
            .get(&dependency.consumer)
            .ok_or(WorkError::UnknownUnit(dependency.consumer))?;
        let producer = units
            .get(&dependency.producer)
            .ok_or(WorkError::UnknownUnit(dependency.producer))?;
        if consumer.scope != producer.scope {
            return Err(WorkError::ForeignScope(producer.unit.unit_id));
        }
        if !pairs.insert((dependency.consumer, dependency.producer)) {
            return Err(WorkError::Duplicate(dependency.consumer.unit_id));
        }
        waits.push((
            WaitNode::Unit(dependency.consumer),
            WaitNode::Unit(dependency.producer),
        ));
    }
    validate_links(graph, &mut waits)?;
    if waits.len() > limits.max_edges.get() as usize
        || waits.len() > STRUCTURAL_LIMITS_V1.max_wait_edges
    {
        return Err(WorkError::InvalidField("graph.edges"));
    }
    reject_cycle(graph.units.iter().map(|u| WaitNode::Unit(u.unit)), &waits)?;
    Ok(ValidatedGraph { graph })
}
