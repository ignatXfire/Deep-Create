//! Exact dependency and result checks; a successful decision is not a commit.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Obligation,
    AcceptedResult,
    EvidenceBinding,
    ArtifactBinding,
    WorkspaceBinding,
    ConnectionConfig,
    VerificationCriteria,
    Policy,
    RootWorkLink,
    ResultPublication,
    PublicationState,
    PublicationBinding,
    OwnerAcceptanceBinding,
    UnitResultBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyKey {
    pub kind: DependencyKind,
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedDependency {
    pub key: DependencyKey,
    pub expected_revision: RowVersion,
    /// Opaque caller-verified content identity; this module does not hash data.
    pub fingerprint: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencySnapshot {
    pub snapshot_id: EntityId,
    pub scope: WorkScope,
    pub unit: UnitRef,
    pub read_set: Vec<VersionedDependency>,
    pub policy_versions: Vec<VersionedDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkUse {
    pub link_id: EntityId,
    pub link_revision_id: EntityId,
    pub selected_publication_id: Option<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStage {
    Submitted,
    Verifying,
    Accepted,
    Rejected,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultPacket {
    pub result_id: EntityId,
    pub attempt_id: EntityId,
    pub scope: WorkScope,
    pub unit: UnitRef,
    pub snapshot: DependencySnapshot,
    pub fingerprint: [u8; 32],
    pub verification_policy: VersionedDependency,
    pub used_links: Vec<LinkUse>,
    pub stage: ResultStage,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationVerdict {
    Pass,
    Gaps,
    Reject,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationFinding {
    pub obligation_id: EntityId,
    pub status: ObligationStatus,
    pub evidence_id: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationStatus {
    Qualified,
    Restricted,
    InsufficientEvidence,
    Stale,
    Disqualified,
}

/// A canonical C-08 qualification decision for a specific method and criteria.
/// References are authenticated by the future adapter; this is NOT calibration
/// computation or proof that arbitrary input data are qualified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodQualification {
    pub qualification_id: EntityId,
    pub method_revision_id: EntityId,
    pub work_kind: WorkKind,
    pub criterion_ids: Vec<EntityId>,
    pub level: VerificationLevel,
    pub policy: VersionedDependency,
    pub status: QualificationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub verification_plan_id: EntityId,
    pub method_revision_id: EntityId,
    pub scope: WorkScope,
    pub work_kind: WorkKind,
    pub criterion_ids: Vec<EntityId>,
    pub level: VerificationLevel,
    /// Minimum captured by the accepted plan, including elevated consequences.
    pub required_level: VerificationLevel,
    pub machine_findings_id: EntityId,
    pub policy: VersionedDependency,
    pub qualification: Option<MethodQualification>,
    pub independence_assessment_id: Option<EntityId>,
    pub protected_admission_evidence_id: Option<EntityId>,
}

/// Canonical C-08 record: exact candidate and qualified-method evidence are
/// structurally bound below. Authenticity and calibration remain C-08 duties.
/// ```compile_fail
/// use dc_domain::work::VerificationRecord;
/// let _: VerificationRecord = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationRecord {
    pub verification_id: EntityId,
    pub result_id: EntityId,
    pub result_fingerprint: [u8; 32],
    pub unit: UnitRef,
    pub snapshot_id: EntityId,
    pub verification_policy: VersionedDependency,
    pub verifier_attempt_id: EntityId,
    pub verdict: VerificationVerdict,
    pub findings: Vec<ObligationFinding>,
    pub evidence: VerificationEvidence,
}

/// Current rows are a coherent caller-supplied canonical view, not arbitrary
/// proposed versions. No global EventSeq participates in result freshness.
pub struct ResultCommitView<'a> {
    pub graph: &'a ValidatedGraph<'a>,
    pub current_rows: &'a [VersionedDependency],
    pub expected_unit_row_version: RowVersion,
    pub expected_result_row_version: RowVersion,
    pub active_attempt_id: EntityId,
}

fn checked_rows(
    rows: &[VersionedDependency],
) -> Result<BTreeMap<DependencyKey, &VersionedDependency>, WorkError> {
    if rows.len() > STRUCTURAL_LIMITS_V1.max_current_dependency_rows {
        return Err(WorkError::InvalidField("dependency_rows"));
    }
    let mut current = BTreeMap::new();
    for row in rows {
        if current.insert(row.key, row).is_some() {
            return Err(WorkError::Duplicate(row.key.entity_id));
        }
    }
    Ok(current)
}

pub fn validate_dependency_snapshot(
    snapshot: &DependencySnapshot,
    current_rows: &[VersionedDependency],
) -> Result<(), WorkError> {
    if snapshot.policy_versions.is_empty()
        || snapshot.policy_versions.len() > STRUCTURAL_LIMITS_V1.max_policy_versions
        || snapshot.read_set.len() > STRUCTURAL_LIMITS_V1.max_snapshot_reads
    {
        return Err(WorkError::InvalidField("dependency_snapshot"));
    }
    if snapshot
        .policy_versions
        .iter()
        .any(|p| p.key.kind != DependencyKind::Policy)
    {
        return Err(WorkError::InvalidField("policy_versions"));
    }
    let mut seen = BTreeSet::new();
    let current = checked_rows(current_rows)?;
    for expected in snapshot.read_set.iter().chain(&snapshot.policy_versions) {
        if !seen.insert(expected.key) {
            return Err(WorkError::Duplicate(expected.key.entity_id));
        }
        if current
            .get(&expected.key)
            .is_none_or(|actual| **actual != *expected)
        {
            return Err(WorkError::StaleInput(expected.key));
        }
    }
    Ok(())
}

/// Obtain graph-owned versions for a commit check; policy/configuration rows
/// must be supplied separately by their canonical owners.
pub fn graph_dependency_rows(graph: &ValidatedGraph<'_>) -> Vec<VersionedDependency> {
    let g = graph.graph;
    let mut rows = Vec::new();
    let mut push = |kind, entity_id, expected_revision, fingerprint| {
        rows.push(VersionedDependency {
            key: DependencyKey { kind, entity_id },
            expected_revision,
            fingerprint,
        })
    };
    for o in g.objectives.iter().flat_map(|o| &o.obligations) {
        push(
            DependencyKind::Obligation,
            o.obligation_id,
            o.row_version,
            None,
        );
    }
    for l in &g.links {
        push(DependencyKind::RootWorkLink, l.link_id, l.row_version, None);
    }
    for p in &g.publications {
        push(
            DependencyKind::ResultPublication,
            p.publication_id,
            RowVersion::MIN,
            Some(p.result_fingerprint),
        );
    }
    for s in &g.publication_states {
        push(
            DependencyKind::PublicationState,
            s.publication_id,
            s.row_version,
            None,
        );
    }
    for b in &g.publication_bindings {
        push(
            DependencyKind::PublicationBinding,
            b.binding_id,
            b.row_version,
            None,
        );
    }
    for a in &g.owner_acceptances {
        push(
            DependencyKind::OwnerAcceptanceBinding,
            a.binding_id,
            a.row_version,
            None,
        );
    }
    for unit in &g.units {
        if let Some(result) = &unit.accepted_result {
            push(
                DependencyKind::UnitResultBinding,
                unit.unit.unit_id,
                unit.row_version,
                None,
            );
            push(
                DependencyKind::AcceptedResult,
                result.result_id,
                result.result_row_version,
                Some(result.fingerprint),
            );
        }
    }
    rows
}

pub(crate) fn verify_record(
    packet: &ResultPacket,
    verification: &VerificationRecord,
) -> Result<(), WorkError> {
    if verification.result_id != packet.result_id
        || verification.result_fingerprint != packet.fingerprint
        || verification.unit != packet.unit
        || verification.snapshot_id != packet.snapshot.snapshot_id
        || verification.verification_policy != packet.verification_policy
        || verification.verifier_attempt_id == packet.attempt_id
        || verification.evidence.scope != packet.scope
        || verification.evidence.policy != packet.verification_policy
    {
        return Err(WorkError::VerificationMismatch);
    }
    if verification.findings.len() > MAX_ITEMS {
        return Err(WorkError::InvalidField("verification.findings"));
    }
    unique_ids(verification.findings.iter().map(|f| f.obligation_id))?;
    if verification.evidence.criterion_ids.is_empty()
        || verification.evidence.criterion_ids.len() > MAX_ITEMS
    {
        return Err(WorkError::VerificationMismatch);
    }
    unique_ids(verification.evidence.criterion_ids.iter().copied())?;
    Ok(())
}

fn same_criteria(a: &[EntityId], b: &[EntityId]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    a == b
}

/// Structural C-08 view only: matching a label is insufficient; semantic pass
/// references an exact recorded qualification and independence assessment.
/// This does not requalify a method or establish currentness: canonical lookup,
/// invalidation and qualification read-set integration remain C-08 work.
/// Restricted methods fail closed until their additional-method path exists.
pub(crate) fn verify_record_for_unit(
    packet: &ResultPacket,
    verification: &VerificationRecord,
    unit: &WorkUnitSpec,
    objective: &ObjectiveContract,
) -> Result<(), WorkError> {
    verify_record(packet, verification)?;
    let evidence = &verification.evidence;
    if unit.unit != packet.unit
        || unit.scope != packet.scope
        || objective.scope != unit.scope
        || evidence.work_kind != unit.kind
        || !same_criteria(&evidence.criterion_ids, &unit.acceptance_criteria)
        || !unit
            .acceptance_criteria
            .iter()
            .all(|id| unit.obligation_ids.contains(id))
    {
        return Err(WorkError::VerificationMismatch);
    }
    // Document 16 section 12.3. An accepted plan may raise this minimum for
    // consequences or owner criteria; it cannot lower it.
    let minimum = match unit.kind {
        WorkKind::DeterministicCheck | WorkKind::Verification => VerificationLevel::MachineOnly,
        WorkKind::Research
        | WorkKind::CreativeExploration
        | WorkKind::Planning
        | WorkKind::Summary => VerificationLevel::IndependentReview,
        WorkKind::Synthesis | WorkKind::PreflightAnalysis | WorkKind::LocalChangeProposal => {
            VerificationLevel::DiverseChallenge
        }
        WorkKind::ModelAdaptation => VerificationLevel::ProtectedAdmission,
    };
    if evidence.level < minimum || evidence.level < evidence.required_level {
        return Err(WorkError::VerificationMismatch);
    }
    if evidence.level == VerificationLevel::MachineOnly {
        if !unit.acceptance_criteria.iter().all(|id| {
            objective.obligations.iter().any(|o| {
                o.obligation_id == *id && o.verification_method == VerificationMethod::Deterministic
            })
        }) {
            return Err(WorkError::VerificationMismatch);
        }
    } else {
        let qualified = evidence
            .qualification
            .as_ref()
            .ok_or(WorkError::VerificationMismatch)?;
        if qualified.criterion_ids.is_empty() || qualified.criterion_ids.len() > MAX_ITEMS {
            return Err(WorkError::VerificationMismatch);
        }
        unique_ids(qualified.criterion_ids.iter().copied())?;
        if qualified.status != QualificationStatus::Qualified
            || qualified.work_kind != unit.kind
            || qualified.method_revision_id != evidence.method_revision_id
            || qualified.level < evidence.level
            || qualified.policy != packet.verification_policy
            || !same_criteria(&qualified.criterion_ids, &unit.acceptance_criteria)
            || evidence.independence_assessment_id.is_none()
        {
            return Err(WorkError::VerificationMismatch);
        }
        if evidence.level == VerificationLevel::ProtectedAdmission
            && evidence.protected_admission_evidence_id.is_none()
        {
            return Err(WorkError::VerificationMismatch);
        }
    }
    for criterion in &unit.acceptance_criteria {
        if !verification
            .findings
            .iter()
            .any(|f| f.obligation_id == *criterion && f.status == ObligationStatus::Satisfied)
        {
            return Err(WorkError::VerificationMismatch);
        }
    }
    if verification.findings.iter().any(|f| {
        !unit.obligation_ids.contains(&f.obligation_id)
            || matches!(
                f.status,
                ObligationStatus::Violated | ObligationStatus::Waived
            )
    }) {
        return Err(WorkError::VerificationMismatch);
    }
    Ok(())
}

fn verify_basis(
    packet: &ResultPacket,
    graph: &ValidatedGraph<'_>,
    current_rows: &[VersionedDependency],
) -> Result<(), WorkError> {
    let unit = graph
        .unit(packet.unit)
        .ok_or(WorkError::ResultBasisMismatch)?;
    let root = graph
        .graph
        .roots
        .iter()
        .find(|r| r.work_id == packet.scope.work_id)
        .ok_or(WorkError::ResultBasisMismatch)?;
    if root.active_objective_id != packet.scope.objective_id
        || unit.scope != packet.scope
        || packet.snapshot.scope != packet.scope
        || packet.snapshot.unit != packet.unit
    {
        return Err(WorkError::ResultBasisMismatch);
    }
    validate_dependency_snapshot(&packet.snapshot, current_rows)?;
    if packet.verification_policy.key.kind != DependencyKind::Policy
        || !packet
            .snapshot
            .policy_versions
            .contains(&packet.verification_policy)
    {
        return Err(WorkError::VerificationMismatch);
    }
    for obligation_id in unit.obligation_ids.iter().chain(&unit.acceptance_criteria) {
        let key = DependencyKey {
            kind: DependencyKind::Obligation,
            entity_id: *obligation_id,
        };
        if !packet.snapshot.read_set.iter().any(|r| r.key == key) {
            return Err(WorkError::IncompleteSnapshot(key));
        }
    }
    for dependency in graph
        .graph
        .dependencies
        .iter()
        .filter(|d| d.consumer == packet.unit)
    {
        if dependency.relation == DependencyRelation::MustAuthorize {
            return Err(WorkError::UnsupportedDependencyRelation(
                dependency.relation,
            ));
        }
        let producer = graph
            .unit(dependency.producer)
            .ok_or(WorkError::ResultBasisMismatch)?;
        let accepted = producer
            .accepted_result
            .as_ref()
            .filter(|_| producer.stage == UnitStage::Accepted)
            .ok_or({
                WorkError::Blocked(BlockingReason::DependencyPending {
                    unit: dependency.producer,
                })
            })?;
        let required = [
            VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::UnitResultBinding,
                    entity_id: producer.unit.unit_id,
                },
                expected_revision: producer.row_version,
                fingerprint: None,
            },
            VersionedDependency {
                key: DependencyKey {
                    kind: DependencyKind::AcceptedResult,
                    entity_id: accepted.result_id,
                },
                expected_revision: accepted.result_row_version,
                fingerprint: Some(accepted.fingerprint),
            },
        ];
        for read in required {
            if !packet.snapshot.read_set.contains(&read) {
                return Err(WorkError::IncompleteSnapshot(read.key));
            }
        }
    }
    if packet.used_links.len() > MAX_ITEMS {
        return Err(WorkError::InvalidField("used_links"));
    }
    unique_ids(packet.used_links.iter().map(|l| l.link_id))?;
    for link in &graph.graph.links {
        if link.relation != RootWorkRelation::Informs
            && target_units(graph.graph, link)?.contains(&packet.unit)
            && !packet.used_links.iter().any(|u| u.link_id == link.link_id)
        {
            return Err(WorkError::IncompleteSnapshot(DependencyKey {
                kind: DependencyKind::RootWorkLink,
                entity_id: link.link_id,
            }));
        }
    }
    for usage in &packet.used_links {
        let link = graph
            .graph
            .links
            .iter()
            .find(|l| l.link_id == usage.link_id)
            .ok_or(WorkError::ResultBasisMismatch)?;
        if link.revision_id != usage.link_revision_id
            || !target_units(graph.graph, link)?.contains(&packet.unit)
        {
            return Err(WorkError::ResultBasisMismatch);
        }
        let assessment = assess_link(graph, usage.link_id)?;
        if let Some(reason) = assessment.blocking_reasons.first() {
            return Err(WorkError::Blocked(reason.clone()));
        }
        if assessment.selected_publication_id != usage.selected_publication_id {
            return Err(WorkError::ResultBasisMismatch);
        }
        for required in assessment.read_set {
            if !packet.snapshot.read_set.contains(&required)
                && !packet.snapshot.policy_versions.contains(&required)
            {
                return Err(WorkError::IncompleteSnapshot(required.key));
            }
        }
    }
    // No caller-supplied version can override graph-owned canonical versions.
    let graph_rows = graph_dependency_rows(graph);
    let graph_keys: BTreeSet<_> = graph_rows.iter().map(|r| r.key).collect();
    for expected in &packet.snapshot.read_set {
        let graph_owned = matches!(
            expected.key.kind,
            DependencyKind::Obligation
                | DependencyKind::RootWorkLink
                | DependencyKind::ResultPublication
                | DependencyKind::PublicationState
                | DependencyKind::PublicationBinding
                | DependencyKind::OwnerAcceptanceBinding
                | DependencyKind::UnitResultBinding
                | DependencyKind::AcceptedResult
        );
        if (graph_owned || graph_keys.contains(&expected.key)) && !graph_rows.contains(expected) {
            return Err(WorkError::StaleInput(expected.key));
        }
    }
    Ok(())
}

/// C-03 must atomically commit this returned replacement with its expected rows.
/// No accepting path exists directly from Submitted, and a failed check mutates
/// neither packet nor view. Stale/rejected evidence remains in the caller's store.
pub fn transition_result(
    packet: &ResultPacket,
    target: ResultStage,
    verification: Option<&VerificationRecord>,
    view: &ResultCommitView<'_>,
) -> Result<ResultPacket, WorkError> {
    if packet.row_version != view.expected_result_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    if packet.attempt_id != view.active_attempt_id {
        return Err(WorkError::ResultBasisMismatch);
    }
    let unit = view
        .graph
        .unit(packet.unit)
        .ok_or(WorkError::ResultBasisMismatch)?;
    if unit.row_version != view.expected_unit_row_version {
        return Err(WorkError::RowVersionConflict);
    }
    let expected_stage = match packet.stage {
        ResultStage::Submitted => UnitStage::ResultSubmitted,
        ResultStage::Verifying => UnitStage::Verifying,
        _ => return Err(WorkError::InvalidTransition),
    };
    if unit.stage != expected_stage {
        return Err(WorkError::InvalidTransition);
    }
    if target == ResultStage::Accepted {
        let root = view
            .graph
            .graph
            .roots
            .iter()
            .find(|r| r.work_id == packet.scope.work_id)
            .ok_or(WorkError::ResultBasisMismatch)?;
        let assessment = evaluate_gates(&root.blocking_reasons, &[GateOperation::SubmitResult])?;
        if let Some(reason) = assessment.blockers().iter().next() {
            return Err(WorkError::Blocked(reason.clone()));
        }
        if matches!(root.stage, RootStage::Draft | RootStage::OwnerAccepted) {
            return Err(WorkError::InvalidTransition);
        }
    }
    match (packet.stage, target) {
        (ResultStage::Submitted, ResultStage::Verifying) => {
            verify_basis(packet, view.graph, view.current_rows)?;
        }
        (ResultStage::Verifying, ResultStage::Accepted | ResultStage::Rejected) => {
            let record = verification.ok_or(WorkError::VerificationMismatch)?;
            verify_record(packet, record)?;
            if (target == ResultStage::Accepted) != (record.verdict == VerificationVerdict::Pass) {
                return Err(WorkError::VerificationMismatch);
            }
            // Method qualification (work kind, criteria, level, independence) applies
            // to BOTH Accepted and Rejected — a Rejected verdict from an unqualified
            // verifier is as invalid as an Accepted one. Only finding semantics differ.
            let objective = view
                .graph
                .graph
                .objectives
                .iter()
                .find(|o| o.scope == unit.scope)
                .ok_or(WorkError::ResultBasisMismatch)?;
            verify_record_for_unit(packet, record, unit, objective)?;
            if target == ResultStage::Accepted {
                verify_basis(packet, view.graph, view.current_rows)?;
                for criterion in &unit.acceptance_criteria {
                    if !record.findings.iter().any(|f| {
                        f.obligation_id == *criterion && f.status == ObligationStatus::Satisfied
                    }) {
                        return Err(WorkError::VerificationMismatch);
                    }
                }
                if record.findings.iter().any(|f| {
                    !unit.obligation_ids.contains(&f.obligation_id)
                        || matches!(
                            f.status,
                            ObligationStatus::Violated | ObligationStatus::Waived
                        )
                }) {
                    return Err(WorkError::VerificationMismatch);
                }
            }
        }
        (ResultStage::Submitted | ResultStage::Verifying, ResultStage::Stale) => {
            match verify_basis(packet, view.graph, view.current_rows) {
                Err(
                    WorkError::StaleInput(_)
                    | WorkError::ResultBasisMismatch
                    | WorkError::Blocked(_),
                ) => {}
                Err(error) => return Err(error),
                Ok(()) => return Err(WorkError::InvalidTransition),
            }
        }
        _ => return Err(WorkError::InvalidTransition),
    }
    let next = packet
        .row_version
        .get()
        .checked_add(1)
        .and_then(RowVersion::new)
        .ok_or(WorkError::VersionOverflow)?;
    let mut replacement = packet.clone();
    replacement.stage = target;
    replacement.row_version = next;
    Ok(replacement)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionGap {
    UnsatisfiedObligation(EntityId),
    MissingAcceptedResult(EntityId),
    InvalidAcceptedResult {
        result_id: EntityId,
        reason: WorkError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionBasis {
    pub result_id: EntityId,
    pub fingerprint: [u8; 32],
    pub verification_id: EntityId,
    pub snapshot_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    candidate_id: EntityId,
    scope: WorkScope,
    results: Vec<CompletionBasis>,
    owner_judgment_obligations: Vec<EntityId>,
}

impl CompletionCandidate {
    pub fn candidate_id(&self) -> EntityId {
        self.candidate_id
    }
    pub fn scope(&self) -> WorkScope {
        self.scope
    }
    pub fn results(&self) -> &[CompletionBasis] {
        &self.results
    }
    pub fn owner_judgment_obligations(&self) -> &[EntityId] {
        &self.owner_judgment_obligations
    }
    pub fn stage(&self) -> RootStage {
        RootStage::CompletionCandidate
    }
}

pub struct AcceptedResultEvidence<'a> {
    pub packet: &'a ResultPacket,
    pub verification: &'a VerificationRecord,
}

pub fn derive_completion_candidate(
    candidate_id: EntityId,
    objective_id: EntityId,
    graph: &ValidatedGraph<'_>,
    results: &[AcceptedResultEvidence<'_>],
    current_rows: &[VersionedDependency],
) -> Result<CompletionCandidate, Vec<CompletionGap>> {
    let Some(objective) = graph
        .graph
        .objectives
        .iter()
        .find(|o| o.scope.objective_id == objective_id)
    else {
        return Err(vec![CompletionGap::UnsatisfiedObligation(objective_id)]);
    };
    if results.len() > MAX_ITEMS {
        return Err(vec![CompletionGap::UnsatisfiedObligation(objective_id)]);
    }
    let mut gaps = Vec::new();
    let mut satisfied = BTreeSet::new();
    let mut basis = Vec::new();
    let mut seen = BTreeSet::new();
    for evidence in results {
        let packet = evidence.packet;
        let binding_matches = graph.unit(packet.unit).is_some_and(|u| {
            u.stage == UnitStage::Accepted
                && u.accepted_result.as_ref().is_some_and(|r| {
                    r.result_id == packet.result_id
                        && r.snapshot_id == packet.snapshot.snapshot_id
                        && r.fingerprint == packet.fingerprint
                        && r.result_row_version == packet.row_version
                })
        });
        let check = if !binding_matches
            || packet.stage != ResultStage::Accepted
            || packet.scope != objective.scope
            || !seen.insert(packet.result_id)
        {
            Err(WorkError::ResultBasisMismatch)
        } else {
            graph
                .unit(packet.unit)
                .ok_or(WorkError::ResultBasisMismatch)
                .and_then(|unit| {
                    verify_record_for_unit(packet, evidence.verification, unit, objective)
                })
                .and_then(|()| {
                    if evidence.verification.verdict != VerificationVerdict::Pass {
                        return Err(WorkError::VerificationMismatch);
                    }
                    verify_basis(packet, graph, current_rows)
                })
        };
        if let Err(reason) = check {
            gaps.push(CompletionGap::InvalidAcceptedResult {
                result_id: packet.result_id,
                reason,
            });
            continue;
        }
        let Some(unit) = graph.unit(packet.unit) else {
            continue;
        };
        for finding in &evidence.verification.findings {
            if matches!(
                finding.status,
                ObligationStatus::Violated | ObligationStatus::Unknown
            ) && unit.obligation_ids.contains(&finding.obligation_id)
            {
                gaps.push(CompletionGap::UnsatisfiedObligation(finding.obligation_id));
            }
            if finding.status == ObligationStatus::Satisfied
                && unit.obligation_ids.contains(&finding.obligation_id)
            {
                satisfied.insert(finding.obligation_id);
            }
        }
        basis.push(CompletionBasis {
            result_id: packet.result_id,
            fingerprint: packet.fingerprint,
            verification_id: evidence.verification.verification_id,
            snapshot_id: packet.snapshot.snapshot_id,
        });
    }
    let mut owner_judgment_obligations = Vec::new();
    for o in &objective.obligations {
        if o.kind == ObligationKind::Preference || o.status == ObligationStatus::Waived {
            continue;
        }
        if o.verification_method == VerificationMethod::OwnerJudgment {
            owner_judgment_obligations.push(o.obligation_id);
            continue;
        }
        if o.status == ObligationStatus::Violated || o.status == ObligationStatus::Unknown {
            gaps.push(CompletionGap::UnsatisfiedObligation(o.obligation_id));
        } else if !satisfied.contains(&o.obligation_id) {
            gaps.push(CompletionGap::MissingAcceptedResult(o.obligation_id));
        }
    }
    if !gaps.is_empty() {
        return Err(gaps);
    }
    basis.sort_by_key(|b| b.result_id);
    owner_judgment_obligations.sort();
    Ok(CompletionCandidate {
        candidate_id,
        scope: objective.scope,
        results: basis,
        owner_judgment_obligations,
    })
}

/// Constructed only after the authenticated owner-event path has resolved an
/// exact final reviewed candidate. No Deserialize implementation is provided.
pub struct OwnerAcceptanceDecision {
    pub owner_event_id: EntityId,
    pub candidate_id: EntityId,
    pub scope: WorkScope,
    pub reviewed_results: Vec<CompletionBasis>,
    pub final_verification_id: EntityId,
    pub final_verdict: VerificationVerdict,
    pub accepted_owner_obligations: Vec<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerAcceptedCandidate {
    pub(crate) candidate_id: EntityId,
    pub(crate) scope: WorkScope,
    pub(crate) owner_event_id: EntityId,
    pub(crate) final_verification_id: EntityId,
}

impl OwnerAcceptedCandidate {
    pub fn candidate_id(&self) -> EntityId {
        self.candidate_id
    }
    pub fn owner_event_id(&self) -> EntityId {
        self.owner_event_id
    }
}

pub fn validate_owner_acceptance(
    candidate: &CompletionCandidate,
    decision: &OwnerAcceptanceDecision,
) -> Result<OwnerAcceptedCandidate, WorkError> {
    let mut obligations = decision.accepted_owner_obligations.clone();
    obligations.sort();
    if decision.candidate_id != candidate.candidate_id
        || decision.scope != candidate.scope
        || decision.reviewed_results != candidate.results
        || decision.final_verdict != VerificationVerdict::Pass
        || obligations != candidate.owner_judgment_obligations
    {
        return Err(WorkError::OwnerAcceptanceMismatch);
    }
    Ok(OwnerAcceptedCandidate {
        candidate_id: candidate.candidate_id,
        scope: candidate.scope,
        owner_event_id: decision.owner_event_id,
        final_verification_id: decision.final_verification_id,
    })
}
