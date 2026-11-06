//! Exact cross-work graph and dependency selection, not data-access permission.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    MachineOnly,
    Semantic,
    IndependentReview,
    DiverseChallenge,
    ProtectedAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarKind {
    Boolean,
    Integer,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CanonicalScalar {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

impl CanonicalScalar {
    fn kind(&self) -> ScalarKind {
        match self {
            Self::Boolean(_) => ScalarKind::Boolean,
            Self::Integer(_) => ScalarKind::Integer,
            Self::Text(_) => ScalarKind::Text,
        }
    }
    fn validate(&self) -> Result<(), WorkError> {
        if let Self::Text(text) = self
            && text.len() > MAX_TEXT_BYTES
        {
            return Err(WorkError::InvalidLinkCondition);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSchema {
    pub field_id: EntityId,
    pub kind: ScalarKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputContract {
    pub export_name: String,
    pub schema_version: NonZeroU32,
    pub fields: Vec<FieldSchema>,
}

pub(crate) fn export_name(name: &str) -> Result<(), WorkError> {
    if name.is_empty()
        || name.len() > STRUCTURAL_LIMITS_V1.max_export_name_bytes
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(WorkError::InvalidField("export_name"));
    }
    Ok(())
}

impl OutputContract {
    pub(crate) fn validate(&self) -> Result<(), WorkError> {
        export_name(&self.export_name)?;
        if self.fields.len() > STRUCTURAL_LIMITS_V1.max_output_fields {
            return Err(WorkError::InvalidField("output.fields"));
        }
        unique_ids(self.fields.iter().map(|f| f.field_id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedField {
    pub field_id: EntityId,
    pub value: CanonicalScalar,
}

/// Explicit frozen provenance for an accepted historical unit revision. A pinned
/// publication is not rebound to the producer's newer current revision.
/// Native C-03/04/08 adapters must authenticate these records and archive bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationSource {
    pub objective: ObjectiveContract,
    pub unit: WorkUnitSpec,
    pub packet: ResultPacket,
    pub verification: VerificationRecord,
    pub output_manifest_evidence_id: EntityId,
    pub accepted_fields: Vec<AcceptedField>,
}

/// Partial canonical catalogue view; not a publication command or a wire DTO.
/// This module checks internal provenance, not the external durable checkpoint.
/// ```compile_fail
/// use dc_domain::work::ResultPublication;
/// let _: ResultPublication = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultPublication {
    pub publication_id: EntityId,
    pub producer_work_id: EntityId,
    pub producer_objective_id: EntityId,
    pub producer_unit: UnitRef,
    pub export_revision: NonZeroU32,
    pub output: OutputContract,
    pub result_id: EntityId,
    pub result_fingerprint: [u8; 32],
    pub verification_id: EntityId,
    pub verification_level: VerificationLevel,
    pub accepted_fields: Vec<AcceptedField>,
    pub source: PublicationSource,
    pub output_manifest_evidence_id: EntityId,
    pub created_event_id: EntityId,
    pub predecessor_publication_id: Option<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationEligibility {
    Valid,
    Unresolved,
    Withdrawn,
}

/// Canonical mutable eligibility, not model input.
/// ```compile_fail
/// use dc_domain::work::PublicationState;
/// let _: PublicationState = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationState {
    pub publication_id: EntityId,
    pub eligibility: PublicationEligibility,
    pub reason_evidence_id: Option<EntityId>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationBinding {
    pub binding_id: EntityId,
    pub producer_work_id: EntityId,
    pub export_name: String,
    pub head_publication_id: EntityId,
    pub row_version: RowVersion,
}

/// Canonical owner-event binding, not model input.
/// ```compile_fail
/// use dc_domain::work::OwnerAcceptanceBinding;
/// let _: OwnerAcceptanceBinding = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnerAcceptanceBinding {
    pub binding_id: EntityId,
    pub producer_objective_id: EntityId,
    pub candidate_id: EntityId,
    pub owner_event_id: EntityId,
    pub included_publication_ids: Vec<EntityId>,
    pub valid: bool,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootWorkRelation {
    RequiresResult,
    ConsumesArtifact,
    Informs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PublicationSelector {
    Pinned {
        publication_id: EntityId,
    },
    FollowHead {
        binding_id: EntityId,
    },
    AwaitOutput {
        producer_unit: UnitRef,
        export_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerAcceptanceRequirement {
    NotRequired,
    RequiredForPublicationObjective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceChangePolicy {
    KeepPinned,
    RequireRevalidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConditionAtom {
    ObligationHasStatus {
        obligation_id: EntityId,
        status: ObligationStatus,
    },
    AcceptedFieldEquals {
        publication_id: EntityId,
        field_id: EntityId,
        value: CanonicalScalar,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LinkCondition {
    Always {},
    All { atoms: Vec<ConditionAtom> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConsumerTargetRef {
    Unit { unit: UnitRef },
    Obligation { obligation_id: EntityId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootWorkLink {
    pub link_id: EntityId,
    pub revision_id: EntityId,
    pub row_version: RowVersion,
    pub producer_work_id: EntityId,
    pub consumer_work_id: EntityId,
    pub consumer_objective_id: EntityId,
    pub consumer_targets: Vec<ConsumerTargetRef>,
    pub relation: RootWorkRelation,
    pub selector: PublicationSelector,
    pub required_output_schema: NonZeroU32,
    pub required_verification_level: VerificationLevel,
    pub owner_acceptance: OwnerAcceptanceRequirement,
    pub condition: LinkCondition,
    pub freshness_policy: VersionedDependency,
    pub on_change: SourceChangePolicy,
    pub accepted_basis_ref: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionResult {
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkAssessment {
    pub condition: ConditionResult,
    pub selected_publication_id: Option<EntityId>,
    /// Canonical read set includes condition reads even when condition is false.
    pub read_set: Vec<VersionedDependency>,
    pub blocking_reasons: Vec<BlockingReason>,
}

fn publication(graph: &GraphCandidate, id: EntityId) -> Result<&ResultPublication, WorkError> {
    graph
        .publications
        .iter()
        .find(|p| p.publication_id == id)
        .ok_or(WorkError::UnknownPublication(id))
}

fn selected<'a>(
    graph: &'a GraphCandidate,
    link: &RootWorkLink,
) -> Result<Option<&'a ResultPublication>, WorkError> {
    match &link.selector {
        PublicationSelector::Pinned { publication_id } => {
            publication(graph, *publication_id).map(Some)
        }
        PublicationSelector::FollowHead { binding_id } => {
            let binding = graph
                .publication_bindings
                .iter()
                .find(|b| b.binding_id == *binding_id)
                .ok_or(WorkError::UnknownPublication(*binding_id))?;
            publication(graph, binding.head_publication_id).map(Some)
        }
        PublicationSelector::AwaitOutput {
            producer_unit,
            export_name,
        } => Ok(graph
            .publications
            .iter()
            .filter(|p| {
                p.producer_unit == *producer_unit
                    && p.producer_work_id == link.producer_work_id
                    && p.output.export_name == *export_name
            })
            .max_by_key(|p| p.export_revision)),
    }
}

fn acceptance<'a>(
    graph: &'a GraphCandidate,
    p: &ResultPublication,
) -> Option<&'a OwnerAcceptanceBinding> {
    graph.owner_acceptances.iter().find(|a| {
        a.valid
            && a.producer_objective_id == p.producer_objective_id
            && a.included_publication_ids.contains(&p.publication_id)
    })
}

pub(crate) fn target_units(
    graph: &GraphCandidate,
    link: &RootWorkLink,
) -> Result<Vec<UnitRef>, WorkError> {
    let objective = graph
        .objectives
        .iter()
        .find(|o| o.scope.objective_id == link.consumer_objective_id)
        .ok_or(WorkError::UnknownObjective(link.consumer_objective_id))?;
    let root = graph
        .roots
        .iter()
        .find(|r| r.work_id == link.consumer_work_id)
        .ok_or(WorkError::ForeignScope(link.consumer_work_id))?;
    if objective.scope.work_id != link.consumer_work_id
        || root.active_objective_id != objective.scope.objective_id
    {
        return Err(WorkError::ForeignScope(link.consumer_objective_id));
    }
    let mut units = BTreeSet::new();
    for target in &link.consumer_targets {
        match target {
            ConsumerTargetRef::Unit { unit } => {
                let actual = graph
                    .units
                    .iter()
                    .find(|u| u.unit == *unit)
                    .ok_or(WorkError::UnknownUnit(*unit))?;
                if actual.scope != objective.scope {
                    return Err(WorkError::ForeignScope(unit.unit_id));
                }
                units.insert(*unit);
            }
            ConsumerTargetRef::Obligation { obligation_id } => {
                if !objective
                    .obligations
                    .iter()
                    .any(|o| o.obligation_id == *obligation_id)
                {
                    return Err(WorkError::UnknownObligation(*obligation_id));
                }
                let matched: Vec<_> = graph
                    .units
                    .iter()
                    .filter(|u| {
                        u.scope == objective.scope && u.obligation_ids.contains(obligation_id)
                    })
                    .map(|u| u.unit)
                    .collect();
                if matched.is_empty() {
                    return Err(WorkError::UnboundObligation(*obligation_id));
                }
                units.extend(matched);
            }
        }
    }
    Ok(units.into_iter().collect())
}

pub(crate) fn validate_links(
    graph: &GraphCandidate,
    waits: &mut Vec<(WaitNode, WaitNode)>,
) -> Result<(), WorkError> {
    unique_ids(graph.publications.iter().map(|p| p.publication_id))?;
    unique_ids(graph.publication_states.iter().map(|p| p.publication_id))?;
    unique_ids(graph.publication_bindings.iter().map(|p| p.binding_id))?;
    unique_ids(graph.owner_acceptances.iter().map(|a| a.binding_id))?;
    unique_ids(graph.links.iter().map(|l| l.link_id))?;
    unique_ids(graph.links.iter().map(|l| l.revision_id))?;
    let mut exports = BTreeSet::new();
    for p in &graph.publications {
        p.output.validate()?;
        validate_publication_source(graph, p)?;
        if !exports.insert((p.producer_work_id, &p.output.export_name, p.export_revision)) {
            return Err(WorkError::Duplicate(p.publication_id));
        }
        if let Some(predecessor_id) = p.predecessor_publication_id {
            let previous = publication(graph, predecessor_id)?;
            if previous.producer_work_id != p.producer_work_id
                || previous.output.export_name != p.output.export_name
                || previous.export_revision >= p.export_revision
            {
                return Err(WorkError::InvalidField("publication.predecessor"));
            }
        }
        if !graph
            .publication_states
            .iter()
            .any(|s| s.publication_id == p.publication_id)
        {
            return Err(WorkError::UnknownPublication(p.publication_id));
        }
        if p.accepted_fields.len() > STRUCTURAL_LIMITS_V1.max_output_fields {
            return Err(WorkError::InvalidField("publication.fields"));
        }
        unique_ids(p.accepted_fields.iter().map(|f| f.field_id))?;
        for field in &p.accepted_fields {
            field.value.validate()?;
            if !p
                .output
                .fields
                .iter()
                .any(|f| f.field_id == field.field_id && f.kind == field.value.kind())
            {
                return Err(WorkError::OutputSchemaMismatch);
            }
        }
    }
    for s in &graph.publication_states {
        publication(graph, s.publication_id)?;
        if s.eligibility != PublicationEligibility::Valid && s.reason_evidence_id.is_none() {
            return Err(WorkError::InvalidField("publication.reason"));
        }
    }
    let mut binding_names = BTreeSet::new();
    for b in &graph.publication_bindings {
        export_name(&b.export_name)?;
        let p = publication(graph, b.head_publication_id)?;
        if p.producer_work_id != b.producer_work_id || p.output.export_name != b.export_name {
            return Err(WorkError::ForeignScope(b.binding_id));
        }
        if !binding_names.insert((b.producer_work_id, &b.export_name)) {
            return Err(WorkError::Duplicate(b.binding_id));
        }
    }
    let mut accepted_objectives = BTreeSet::new();
    for a in &graph.owner_acceptances {
        if a.included_publication_ids.len() > MAX_ITEMS {
            return Err(WorkError::InvalidField("acceptance.publications"));
        }
        unique_ids(a.included_publication_ids.iter().copied())?;
        if a.valid && !accepted_objectives.insert(a.producer_objective_id) {
            return Err(WorkError::Duplicate(a.producer_objective_id));
        }
        for id in &a.included_publication_ids {
            if publication(graph, *id)?.producer_objective_id != a.producer_objective_id {
                return Err(WorkError::ForeignScope(*id));
            }
        }
    }
    for link in &graph.links {
        if link.consumer_targets.is_empty()
            || link.consumer_targets.len() > STRUCTURAL_LIMITS_V1.max_link_targets
        {
            return Err(WorkError::InvalidField("link.targets"));
        }
        if link.producer_work_id == link.consumer_work_id
            || !graph
                .roots
                .iter()
                .any(|r| r.work_id == link.producer_work_id)
        {
            return Err(WorkError::ForeignScope(link.producer_work_id));
        }
        if link.freshness_policy.key.kind != DependencyKind::Policy {
            return Err(WorkError::UnsupportedLinkPolicy);
        }
        if matches!(link.selector, PublicationSelector::FollowHead { .. })
            && link.on_change != SourceChangePolicy::RequireRevalidation
        {
            return Err(WorkError::UnsupportedLinkPolicy);
        }
        let consumers = target_units(graph, link)?;
        let p = selected(graph, link)?;
        if let Some(p) = p {
            if p.producer_work_id != link.producer_work_id {
                return Err(WorkError::ForeignScope(p.publication_id));
            }
            if p.output.schema_version != link.required_output_schema
                || p.verification_level < link.required_verification_level
            {
                return Err(WorkError::OutputSchemaMismatch);
            }
        }
        if let PublicationSelector::AwaitOutput {
            producer_unit,
            export_name: name,
        } = &link.selector
        {
            export_name(name)?;
            let unit = graph
                .units
                .iter()
                .find(|u| u.unit == *producer_unit)
                .ok_or(WorkError::UnknownUnit(*producer_unit))?;
            if unit.scope.work_id != link.producer_work_id {
                return Err(WorkError::ForeignScope(producer_unit.unit_id));
            }
            if unit.expected_output.export_name != *name
                || unit.expected_output.schema_version != link.required_output_schema
            {
                return Err(WorkError::OutputSchemaMismatch);
            }
            if p.is_none() && link.relation != RootWorkRelation::Informs {
                for consumer in &consumers {
                    waits.push((WaitNode::Unit(*consumer), WaitNode::Unit(*producer_unit)));
                }
            }
        }
        if let LinkCondition::All { atoms } = &link.condition {
            if atoms.is_empty() || atoms.len() > STRUCTURAL_LIMITS_V1.max_condition_atoms {
                return Err(WorkError::InvalidLinkCondition);
            }
            for atom in atoms {
                match atom {
                    ConditionAtom::ObligationHasStatus { obligation_id, .. } => {
                        let o = graph
                            .objectives
                            .iter()
                            .flat_map(|o| &o.obligations)
                            .find(|o| o.obligation_id == *obligation_id)
                            .ok_or(WorkError::InvalidLinkCondition)?;
                        if !graph.objectives.iter().any(|obj| {
                            obj.scope.objective_id == o.objective_id
                                && (obj.scope.work_id == link.consumer_work_id
                                    || obj.scope.work_id == link.producer_work_id)
                        }) {
                            return Err(WorkError::InvalidLinkCondition);
                        }
                    }
                    ConditionAtom::AcceptedFieldEquals {
                        publication_id,
                        field_id,
                        value,
                    } => {
                        value.validate()?;
                        let p = publication(graph, *publication_id)?;
                        if p.producer_work_id != link.producer_work_id
                            || !p
                                .output
                                .fields
                                .iter()
                                .any(|f| f.field_id == *field_id && f.kind == value.kind())
                        {
                            return Err(WorkError::InvalidLinkCondition);
                        }
                    }
                }
            }
        }
        if link.owner_acceptance == OwnerAcceptanceRequirement::RequiredForPublicationObjective
            && link.relation != RootWorkRelation::Informs
            && p.is_none_or(|p| acceptance(graph, p).is_none())
        {
            let objective_id = match p {
                Some(p) => p.producer_objective_id,
                None => match &link.selector {
                    PublicationSelector::AwaitOutput { producer_unit, .. } => {
                        graph
                            .units
                            .iter()
                            .find(|u| u.unit == *producer_unit)
                            .ok_or(WorkError::UnknownUnit(*producer_unit))?
                            .scope
                            .objective_id
                    }
                    _ => return Err(WorkError::UnsupportedLinkPolicy),
                },
            };
            let node = WaitNode::OwnerAcceptance(objective_id);
            for consumer in consumers {
                waits.push((WaitNode::Unit(consumer), node));
            }
            // Completion of this exact objective cannot precede its unfinished units.
            for unit in graph
                .units
                .iter()
                .filter(|u| u.scope.objective_id == objective_id && u.stage != UnitStage::Accepted)
            {
                waits.push((node, WaitNode::Unit(unit.unit)));
            }
        }
    }
    Ok(())
}

fn validate_publication_source(
    graph: &GraphCandidate,
    p: &ResultPublication,
) -> Result<(), WorkError> {
    let source = &p.source;
    let unit = &source.unit;
    let packet = &source.packet;
    validate_objective(&source.objective, &OwnerConstraints::default())?;
    let root = graph
        .roots
        .iter()
        .find(|r| r.work_id == p.producer_work_id)
        .ok_or(WorkError::ForeignScope(p.producer_work_id))?;
    if source.objective.scope.work_id != p.producer_work_id
        || source.objective.scope.objective_id != p.producer_objective_id
        || source.objective.scope.session_id != root.session_id
        || unit.scope != source.objective.scope
        || unit.unit != p.producer_unit
        || packet.unit != unit.unit
        || packet.scope != unit.scope
        || packet.snapshot.scope != unit.scope
        || packet.snapshot.unit != unit.unit
    {
        return Err(WorkError::ForeignScope(p.publication_id));
    }
    if unit.expected_output != p.output
        || p.accepted_fields != source.accepted_fields
        || p.output_manifest_evidence_id != source.output_manifest_evidence_id
    {
        return Err(WorkError::OutputSchemaMismatch);
    }
    let accepted = unit
        .accepted_result
        .as_ref()
        .ok_or(WorkError::ResultBasisMismatch)?;
    if unit.stage != UnitStage::Accepted
        || packet.stage != ResultStage::Accepted
        || accepted.result_id != packet.result_id
        || accepted.snapshot_id != packet.snapshot.snapshot_id
        || accepted.fingerprint != packet.fingerprint
        || accepted.result_row_version != packet.row_version
        || p.result_id != packet.result_id
        || p.result_fingerprint != packet.fingerprint
        || p.verification_id != source.verification.verification_id
        || source.verification.verdict != VerificationVerdict::Pass
        || p.verification_level != source.verification.evidence.level
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    if unit.obligation_ids.is_empty()
        || unit.obligation_ids.len() > MAX_ITEMS
        || unit.acceptance_criteria.len() > MAX_ITEMS
    {
        return Err(WorkError::MissingCriterion);
    }
    unique_ids(unit.obligation_ids.iter().copied())?;
    unique_ids(unit.acceptance_criteria.iter().copied())?;
    if !unit.obligation_ids.iter().all(|id| {
        source
            .objective
            .obligations
            .iter()
            .any(|o| o.obligation_id == *id)
    }) {
        return Err(WorkError::ForeignScope(p.publication_id));
    }
    verify_record_for_unit(packet, &source.verification, unit, &source.objective)?;
    // A retained source revision may differ from the current head. If the exact
    // revision is current, its immutable task contract must still agree.
    if let Some(current) = graph.units.iter().find(|u| u.unit == unit.unit)
        && (current.scope != unit.scope
            || current.kind != unit.kind
            || current.parent != unit.parent
            || current.expected_output != unit.expected_output
            || current.obligation_ids != unit.obligation_ids
            || current.acceptance_criteria != unit.acceptance_criteria
            || current.bounds != unit.bounds
            || current.exploration != unit.exploration)
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    Ok(())
}

fn read(
    kind: DependencyKind,
    id: EntityId,
    version: RowVersion,
    fingerprint: Option<[u8; 32]>,
) -> VersionedDependency {
    VersionedDependency {
        key: DependencyKey {
            kind,
            entity_id: id,
        },
        expected_revision: version,
        fingerprint,
    }
}

fn read_publication(
    graph: &GraphCandidate,
    p: &ResultPublication,
    reads: &mut Vec<VersionedDependency>,
) -> Result<PublicationEligibility, WorkError> {
    let s = graph
        .publication_states
        .iter()
        .find(|s| s.publication_id == p.publication_id)
        .ok_or(WorkError::UnknownPublication(p.publication_id))?;
    reads.push(read(
        DependencyKind::ResultPublication,
        p.publication_id,
        RowVersion::MIN,
        Some(p.result_fingerprint),
    ));
    reads.push(read(
        DependencyKind::PublicationState,
        p.publication_id,
        s.row_version,
        None,
    ));
    Ok(s.eligibility)
}

/// Explains selection only. Informs does not block scheduling, but a caller that
/// actually reads it must include this read set and honor its blocked state.
pub fn assess_link(
    graph: &ValidatedGraph<'_>,
    link_id: EntityId,
) -> Result<LinkAssessment, WorkError> {
    let graph = graph.graph;
    let link = graph
        .links
        .iter()
        .find(|l| l.link_id == link_id)
        .ok_or(WorkError::InvalidField("link_id"))?;
    let mut reads = vec![
        read(
            DependencyKind::RootWorkLink,
            link.link_id,
            link.row_version,
            None,
        ),
        link.freshness_policy.clone(),
    ];
    let mut condition = ConditionResult::True;
    if let LinkCondition::All { atoms } = &link.condition {
        for atom in atoms {
            let result = match atom {
                ConditionAtom::ObligationHasStatus {
                    obligation_id,
                    status,
                } => {
                    let o = graph
                        .objectives
                        .iter()
                        .flat_map(|o| &o.obligations)
                        .find(|o| o.obligation_id == *obligation_id)
                        .ok_or(WorkError::InvalidLinkCondition)?;
                    reads.push(read(
                        DependencyKind::Obligation,
                        o.obligation_id,
                        o.row_version,
                        None,
                    ));
                    if o.status == ObligationStatus::Unknown {
                        ConditionResult::Unknown
                    } else if o.status == *status {
                        ConditionResult::True
                    } else {
                        ConditionResult::False
                    }
                }
                ConditionAtom::AcceptedFieldEquals {
                    publication_id,
                    field_id,
                    value,
                } => {
                    let p = publication(graph, *publication_id)?;
                    let eligible = read_publication(graph, p, &mut reads)?;
                    match p.accepted_fields.iter().find(|f| f.field_id == *field_id) {
                        Some(field) if eligible == PublicationEligibility::Valid => {
                            if field.value == *value {
                                ConditionResult::True
                            } else {
                                ConditionResult::False
                            }
                        }
                        _ => ConditionResult::Unknown,
                    }
                }
            };
            // Unknown is fail-closed even if a different atom is false.
            condition = match (condition, result) {
                (ConditionResult::Unknown, _) | (_, ConditionResult::Unknown) => {
                    ConditionResult::Unknown
                }
                (ConditionResult::False, _) | (_, ConditionResult::False) => ConditionResult::False,
                _ => ConditionResult::True,
            };
        }
    }
    let mut assessment = LinkAssessment {
        condition,
        selected_publication_id: None,
        read_set: reads,
        blocking_reasons: Vec::new(),
    };
    if condition == ConditionResult::Unknown {
        assessment
            .blocking_reasons
            .push(BlockingReason::ConditionUnknown { link_id });
        return Ok(assessment);
    }
    if condition == ConditionResult::False {
        return Ok(assessment);
    }
    let Some(p) = selected(graph, link)? else {
        let reason = match &link.selector {
            PublicationSelector::AwaitOutput { producer_unit, .. }
                if graph.units.iter().any(|u| {
                    u.unit == *producer_unit
                        && matches!(u.stage, UnitStage::Cancelled | UnitStage::Cancelling)
                }) =>
            {
                BlockingReason::UpstreamCancelled {
                    unit: *producer_unit,
                }
            }
            _ => BlockingReason::AwaitingPublication { link_id },
        };
        assessment.blocking_reasons.push(reason);
        return Ok(assessment);
    };
    assessment.selected_publication_id = Some(p.publication_id);
    match read_publication(graph, p, &mut assessment.read_set)? {
        PublicationEligibility::Valid => {}
        PublicationEligibility::Unresolved => {
            assessment
                .blocking_reasons
                .push(BlockingReason::PublicationUnresolved {
                    publication_id: p.publication_id,
                })
        }
        PublicationEligibility::Withdrawn => {
            assessment
                .blocking_reasons
                .push(BlockingReason::PublicationWithdrawn {
                    publication_id: p.publication_id,
                })
        }
    }
    if link.on_change == SourceChangePolicy::RequireRevalidation {
        let binding = graph
            .publication_bindings
            .iter()
            .find(|b| {
                b.producer_work_id == p.producer_work_id && b.export_name == p.output.export_name
            })
            .ok_or(WorkError::UnsupportedLinkPolicy)?;
        assessment.read_set.push(read(
            DependencyKind::PublicationBinding,
            binding.binding_id,
            binding.row_version,
            None,
        ));
    }
    if link.owner_acceptance == OwnerAcceptanceRequirement::RequiredForPublicationObjective {
        if let Some(a) = acceptance(graph, p) {
            assessment.read_set.push(read(
                DependencyKind::OwnerAcceptanceBinding,
                a.binding_id,
                a.row_version,
                None,
            ));
        } else {
            assessment
                .blocking_reasons
                .push(BlockingReason::OwnerAcceptancePending {
                    publication_id: p.publication_id,
                });
        }
    }
    let mut canonical = BTreeMap::new();
    for r in assessment.read_set {
        if let Some(old) = canonical.insert(r.key, r.clone())
            && old != r
        {
            return Err(WorkError::InvalidField("conflicting_read_set"));
        }
    }
    assessment.read_set = canonical.into_values().collect();
    Ok(assessment)
}
