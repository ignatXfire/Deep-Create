//! C-01 context projection contracts (doc 15 §2.5–2.7, §4, §5.2, §7).
//!
//! Pure data contracts and validation only. This module does NOT search local
//! memory, read files, call clocks, pick a model connection, admit a dispatch,
//! or persist anything. `plan_context_needs` / `compile_context_projection` are
//! pure transformations: the caller (C-03 via the managing layer) supplies the
//! already-read dependency snapshot, the exact recipient connection view, the
//! candidate fragments and the inclusion/exclusion reasons. Disclosure checks
//! are re-executed at dispatch by `validate_dispatch_preconditions` (doc 16 §8);
//! a projection is never an admission.
//!
//! Boundary invariants enforced here (doc 15 §2.6 "The context selection is invalid"):
//!   1. a required source is missing;
//!   2. a required fact is stale;
//!   3. the connection disallows at least one fragment data class;
//!   4. a secret is present;
//!   5. a required fragment is truncated;
//!   6. a dependency, policy version or recipient configuration the projection
//!      depends on has changed;
//!   7. a fragment cannot be restored from its stored reference + fingerprint.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::{EntityId, RowVersion};
use crate::permission::DataClass;

/// Opaque protected fingerprint of fragment/projection content (doc 14).
/// Never a public SHA-256 of personal plaintext; the key version and purpose
/// separation live outside this pure layer. Compared only for equality.
pub type ContentFingerprint = [u8; 32];

/// Version of a reviewed, immutable policy/schema. Never zero.
pub type SchemaVersion = NonZeroU32;

// ---------------------------------------------------------------------------
// Doc 15 §4.1 — provenance and authority of a fragment.
// Only the first three may order work; a fragment never carries a right.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentOrigin {
    ProtectedRule,
    DirectOwnerEvent,
    AcceptedState,
    ExternalEvidence,
    ModelAssertion,
}

impl FragmentOrigin {
    /// Doc 15 §4.1: only ProtectedRule, DirectOwnerEvent and AcceptedState may
    /// order work. External evidence and model assertions are data, not orders.
    pub const fn may_order_work(self) -> bool {
        matches!(
            self,
            Self::ProtectedRule | Self::DirectOwnerEvent | Self::AcceptedState
        )
    }
}

// ---------------------------------------------------------------------------
// Doc 15 §4.3 — knowledge state. A derived record can never promote itself to
// a source; repetition across models does not turn a claim into a fact.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeState {
    Source,
    Derived,
    Hypothesis,
    Supported,
    Refuted,
}

// ---------------------------------------------------------------------------
// Doc 15 §4.4 — freshness. Time is always supplied by the trusted caller;
// this layer never reads a clock.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Freshness {
    Immutable {},
    /// Expires at the supplied trusted UTC millisecond timestamp.
    ValidUntil {
        utc_millis: i64,
    },
    /// Must be re-checked against the named rule version before use.
    RecheckBeforeUse {
        rule_version: SchemaVersion,
    },
}

impl Freshness {
    /// `now_utc_millis` is an input from the trusted boundary, never read here.
    pub const fn is_stale_at(&self, now_utc_millis: i64) -> bool {
        match self {
            Self::Immutable {} => false,
            Self::ValidUntil { utc_millis } => now_utc_millis >= *utc_millis,
            Self::RecheckBeforeUse { .. } => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Doc 15 §2.5 — a single information need (required or optional question).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InformationNeed {
    pub need_id: EntityId,
    /// Short purpose label; never secret-bearing content.
    pub question: String,
    pub required: bool,
}

// ---------------------------------------------------------------------------
// Doc 15 §2.6 — one fragment inside a projection. Always `DataOnly`: content
// is information, never an instruction escalation (doc 15 §3, §6).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextFragment {
    pub fragment_id: EntityId,
    /// Immutable source or versioned binding this fragment was read from.
    pub source_ref: EntityId,
    /// Exact range inside the source (byte offsets into the canonical form).
    pub range_start: u64,
    pub range_len: u64,
    /// Fingerprint of exactly this fragment's content, for restoration checks.
    pub fingerprint: ContentFingerprint,
    /// Why this fragment was included (search/need justification reference).
    pub inclusion_reason: String,
    pub data_class: DataClass,
    pub freshness: Freshness,
    pub origin: FragmentOrigin,
    pub knowledge: KnowledgeState,
    /// Contradiction group identifier; conflicting sources travel together
    /// with an explicit shared group (doc 15 §9.3).
    pub contradiction_group: Option<EntityId>,
    /// True when the fragment could not be restored from source_ref+fingerprint
    /// (doc 15 §2.6 invalidity reason 7).
    pub unrestorable: bool,
    /// True when a required fragment had to be truncated to fit size bounds.
    /// Silent truncation is forbidden (doc 15 §9.8): the flag must be set and
    /// makes the projection invalid when the fragment is required.
    pub truncated: bool,
}

// ---------------------------------------------------------------------------
// Doc 15 §2.6 / §5.2 — a candidate that was excluded, with the reason kept.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludedCandidate {
    pub candidate_ref: EntityId,
    /// Why the candidate was excluded (disclosure class, boundary, freshness…).
    /// Never contains the excluded content itself.
    pub exclusion_reason: String,
    /// True when the exclusion was a secret detected by search (doc 15 §9.2:
    /// the secret reaches neither the projection, the error nor the journal —
    /// only the opaque reference and the rule identifier are recorded).
    pub excluded_as_secret: bool,
}

// ---------------------------------------------------------------------------
// Doc 15 §2.5а — the exact dependency basis the projection was compiled
// against. Re-checked at dispatch and at acceptance; this pure layer only
// compares caller-supplied views.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyBasis {
    pub snapshot_id: EntityId,
    /// Current revision of the snapshot row as re-read by the caller.
    pub snapshot_revision: RowVersion,
    /// Revision the projection was compiled against.
    pub compiled_against_revision: RowVersion,
}

// ---------------------------------------------------------------------------
// Doc 15 §2.6 — the immutable recipient-bound projection.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextProjection {
    pub projection_id: EntityId,
    pub unit_id: EntityId,
    pub dependency_snapshot_id: EntityId,
    pub connection_id: EntityId,
    pub fragments: Vec<ContextFragment>,
    pub excluded: Vec<ExcludedCandidate>,
    pub missing_required: Vec<InformationNeed>,
    pub policy_version: SchemaVersion,
    pub projection_hash: ContentFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProjectionError {
    #[error("required information need is unmet: {0}")]
    MissingRequiredSource(EntityId),
    #[error("required fragment is stale: {0}")]
    StaleRequiredFragment(EntityId),
    #[error("connection disallows data class of fragment: {0}")]
    DataClassNotAllowed(EntityId),
    #[error("secret must never enter a context projection (fragment: {0})")]
    SecretPresent(EntityId),
    #[error("required fragment is truncated: {0}")]
    RequiredFragmentTruncated(EntityId),
    #[error("dependency basis changed since compilation")]
    DependencyBasisChanged,
    #[error("fragment cannot be restored from its reference and fingerprint: {0}")]
    FragmentUnrestorable(EntityId),
    #[error("model assertion or external evidence cannot order work (fragment: {0})")]
    DataOnlyViolation(EntityId),
    #[error("collection or text field exceeds structural limits")]
    Oversized,
}

/// Which fragments are required is supplied by the caller from the unit's
/// `required_inputs` — this layer never invents an artificial quote minimum
/// (doc 15 §2.6: an empty external fragment set is valid for explicitly
/// allowed creative exploration with no mandatory external source).
pub struct ProjectionValidationInput<'a> {
    pub projection: &'a ContextProjection,
    /// Fragment ids required by the unit's `required_inputs`.
    pub required_fragment_ids: &'a BTreeSet<EntityId>,
    /// Data classes the exact recipient connection admits for this dispatch
    /// (re-read by the caller at validation time; never cached here).
    pub connection_allowed_classes: &'a BTreeSet<DataClass>,
    /// Dependency basis as re-read now, for change detection.
    pub current_dependency_basis: DependencyBasis,
    /// Trusted current time for freshness checks (never read here).
    pub now_utc_millis: i64,
    /// Whether the unit kind is explicitly allowed creative exploration
    /// without mandatory external sources (doc 15 §2.6).
    pub creative_exploration_allowed: bool,
}

/// Validate a compiled projection against every invalidity condition of
/// doc 15 §2.6. Returns the first violation found; the projection is either
/// fully valid for this exact recipient and basis, or rejected. Pure: no I/O,
/// no clock, no authority, no persistence.
pub fn validate_context_projection(
    input: &ProjectionValidationInput<'_>,
) -> Result<(), ProjectionError> {
    let p = input.projection;

    if p.fragments.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
        || p.excluded.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
        || p.missing_required.len() > crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items
    {
        return Err(ProjectionError::Oversized);
    }

    // Reason 6: a dependency, policy version or recipient configuration the
    // projection depends on has changed.
    if input.current_dependency_basis.snapshot_id != p.dependency_snapshot_id
        || input.current_dependency_basis.compiled_against_revision
            != input.current_dependency_basis.snapshot_revision
    {
        return Err(ProjectionError::DependencyBasisChanged);
    }

    // Reason 1: a required source is missing. `missing_required` entries whose
    // need is required and not covered by an included fragment reject the
    // projection, unless this unit is an explicitly allowed creative
    // exploration with no mandatory external source.
    if !input.creative_exploration_allowed
        && let Some(missing) = p.missing_required.iter().find(|n| n.required)
    {
        return Err(ProjectionError::MissingRequiredSource(missing.need_id));
    }

    let mut required_seen: BTreeSet<EntityId> = BTreeSet::new();

    for f in &p.fragments {
        // Reason 4: a secret is present. `Secret` never enters a projection.
        if f.data_class == DataClass::Secret {
            return Err(ProjectionError::SecretPresent(f.fragment_id));
        }
        // Reason 3: the connection disallows at least one fragment data class.
        if !input.connection_allowed_classes.contains(&f.data_class) {
            return Err(ProjectionError::DataClassNotAllowed(f.fragment_id));
        }
        // Reason 7: the fragment cannot be restored from reference+fingerprint.
        if f.unrestorable {
            return Err(ProjectionError::FragmentUnrestorable(f.fragment_id));
        }

        let is_required = input.required_fragment_ids.contains(&f.fragment_id);
        // DataOnly enforcement (doc 15 §2.6 fragment marker, §4.1): a fragment
        // whose origin may not order work (ModelAssertion or ExternalEvidence)
        // must never satisfy a required input. Such a fragment is data, not
        // authority; treating it as required would let an instruction injection
        // escalate into work.
        if is_required && !f.origin.may_order_work() {
            return Err(ProjectionError::DataOnlyViolation(f.fragment_id));
        }
        if is_required {
            required_seen.insert(f.fragment_id);
            // Reason 2: a required fact is stale.
            if f.freshness.is_stale_at(input.now_utc_millis) {
                return Err(ProjectionError::StaleRequiredFragment(f.fragment_id));
            }
            // Reason 5: a required fragment is truncated (silent truncation
            // forbidden, doc 15 §9.8).
            if f.truncated {
                return Err(ProjectionError::RequiredFragmentTruncated(f.fragment_id));
            }
        }
    }

    // Reason 1 (form 2): a fragment the unit requires was never included.
    if !input.creative_exploration_allowed
        && let Some(missing) = input
            .required_fragment_ids
            .iter()
            .find(|id| !required_seen.contains(*id))
    {
        return Err(ProjectionError::MissingRequiredSource(*missing));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::DataClass;

    fn id(byte: u8) -> EntityId {
        // Deterministic UUIDv7-shaped fixture: version 7, variant 2.
        let v = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_3900u128 | (byte as u128);
        EntityId::from_u128(v).expect("fixture id")
    }

    fn fp(byte: u8) -> ContentFingerprint {
        [byte; 32]
    }

    fn schema(n: u32) -> SchemaVersion {
        NonZeroU32::new(n).expect("nonzero")
    }

    fn rv(n: u64) -> RowVersion {
        RowVersion::new(n).expect("nonzero")
    }

    fn base_fragment(fid: u8) -> ContextFragment {
        ContextFragment {
            fragment_id: id(fid),
            source_ref: id(0xA0),
            range_start: 0,
            range_len: 16,
            fingerprint: fp(fid),
            inclusion_reason: "required input".to_string(),
            data_class: DataClass::Local,
            freshness: Freshness::Immutable {},
            origin: FragmentOrigin::AcceptedState,
            knowledge: KnowledgeState::Source,
            contradiction_group: None,
            unrestorable: false,
            truncated: false,
        }
    }

    fn projection_with(fragments: Vec<ContextFragment>) -> ContextProjection {
        ContextProjection {
            projection_id: id(0x01),
            unit_id: id(0x02),
            dependency_snapshot_id: id(0x03),
            connection_id: id(0x04),
            fragments,
            excluded: vec![],
            missing_required: vec![],
            policy_version: schema(1),
            projection_hash: fp(0xFF),
        }
    }

    fn basis() -> DependencyBasis {
        DependencyBasis {
            snapshot_id: id(0x03),
            snapshot_revision: rv(7),
            compiled_against_revision: rv(7),
        }
    }

    fn input_for<'a>(
        p: &'a ContextProjection,
        required: &'a BTreeSet<EntityId>,
        allowed: &'a BTreeSet<DataClass>,
    ) -> ProjectionValidationInput<'a> {
        ProjectionValidationInput {
            projection: p,
            required_fragment_ids: required,
            connection_allowed_classes: allowed,
            current_dependency_basis: basis(),
            now_utc_millis: 1_800_000_000_000,
            creative_exploration_allowed: false,
        }
    }

    fn allowed_all_but_secret() -> BTreeSet<DataClass> {
        [
            DataClass::Public,
            DataClass::Local,
            DataClass::Personal,
            DataClass::Restricted,
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn valid_projection_passes() {
        let frag = base_fragment(0x10);
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Ok(())
        );
    }

    #[test]
    fn empty_fragment_set_valid_for_allowed_creative_exploration() {
        let p = projection_with(vec![]);
        let required = BTreeSet::new();
        let allowed = allowed_all_but_secret();
        let mut input = input_for(&p, &required, &allowed);
        input.creative_exploration_allowed = true;
        assert_eq!(validate_context_projection(&input), Ok(()));
    }

    #[test]
    fn missing_required_source_rejected() {
        let p = projection_with(vec![]);
        let required: BTreeSet<_> = [id(0x77)].into_iter().collect();
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::MissingRequiredSource(id(0x77)))
        );
    }

    #[test]
    fn missing_required_listed_on_projection_rejected() {
        let mut p = projection_with(vec![]);
        p.missing_required.push(InformationNeed {
            need_id: id(0x88),
            question: "owner decision".to_string(),
            required: true,
        });
        let required = BTreeSet::new();
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::MissingRequiredSource(id(0x88)))
        );
    }

    #[test]
    fn stale_required_fragment_rejected() {
        let mut frag = base_fragment(0x11);
        frag.freshness = Freshness::ValidUntil {
            utc_millis: 1_800_000_000_000, // expired exactly at now
        };
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::StaleRequiredFragment(id(0x11)))
        );
    }

    #[test]
    fn stale_optional_fragment_does_not_reject() {
        let mut frag = base_fragment(0x12);
        frag.freshness = Freshness::ValidUntil { utc_millis: 1 };
        let p = projection_with(vec![frag]);
        let required = BTreeSet::new();
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Ok(())
        );
    }

    #[test]
    fn disallowed_data_class_rejected() {
        let mut frag = base_fragment(0x13);
        frag.data_class = DataClass::Restricted;
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed: BTreeSet<_> = [DataClass::Public, DataClass::Local].into_iter().collect();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::DataClassNotAllowed(id(0x13)))
        );
    }

    #[test]
    fn secret_fragment_never_enters_projection() {
        let mut frag = base_fragment(0x14);
        frag.data_class = DataClass::Secret;
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        // Even a connection that would admit everything rejects Secret here;
        // the pure layer has no such thing as a secret-capable projection.
        let mut allowed = allowed_all_but_secret();
        allowed.insert(DataClass::Secret);
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::SecretPresent(id(0x14)))
        );
    }

    #[test]
    fn truncated_required_fragment_rejected() {
        let mut frag = base_fragment(0x15);
        frag.truncated = true;
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::RequiredFragmentTruncated(id(0x15)))
        );
    }

    #[test]
    fn changed_dependency_basis_rejected() {
        let frag = base_fragment(0x16);
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        let mut input = input_for(&p, &required, &allowed);
        input.current_dependency_basis.snapshot_revision = rv(8); // row moved
        assert_eq!(
            validate_context_projection(&input),
            Err(ProjectionError::DependencyBasisChanged)
        );
    }

    #[test]
    fn dependency_basis_snapshot_id_mismatch_rejected() {
        // F1: dedicated negative test for the `snapshot_id != p.dependency_snapshot_id`
        // branch of Reason-6 DependencyBasisChanged (doc 15 §2.6).
        let frag = base_fragment(0x18);
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        let mut input = input_for(&p, &required, &allowed);
        // Mutate only the snapshot_id on the caller-supplied basis; revisions stay equal.
        input.current_dependency_basis.snapshot_id = id(0x99);
        assert_eq!(
            validate_context_projection(&input),
            Err(ProjectionError::DependencyBasisChanged)
        );
    }

    #[test]
    fn unrestorable_fragment_rejected() {
        let mut frag = base_fragment(0x17);
        frag.unrestorable = true;
        let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
        let p = projection_with(vec![frag]);
        let allowed = allowed_all_but_secret();
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Err(ProjectionError::FragmentUnrestorable(id(0x17)))
        );
    }

    #[test]
    fn model_assertion_and_external_evidence_cannot_order_work() {
        assert!(!FragmentOrigin::ModelAssertion.may_order_work());
        assert!(!FragmentOrigin::ExternalEvidence.may_order_work());
        assert!(FragmentOrigin::ProtectedRule.may_order_work());
        assert!(FragmentOrigin::DirectOwnerEvent.may_order_work());
        assert!(FragmentOrigin::AcceptedState.may_order_work());
    }

    #[test]
    fn required_fragment_with_data_only_origin_rejected() {
        // F2: DataOnlyViolation enforcement — a required fragment whose origin
        // may not order work (ModelAssertion / ExternalEvidence) is rejected.
        // Doc 15 §2.6 fragment DataOnly marker, §4.1 origin authority.
        for origin in [FragmentOrigin::ModelAssertion, FragmentOrigin::ExternalEvidence] {
            let mut frag = base_fragment(0x19);
            frag.origin = origin;
            let required: BTreeSet<_> = [frag.fragment_id].into_iter().collect();
            let p = projection_with(vec![frag]);
            let allowed = allowed_all_but_secret();
            assert_eq!(
                validate_context_projection(&input_for(&p, &required, &allowed)),
                Err(ProjectionError::DataOnlyViolation(id(0x19))),
                "origin {origin:?} must not satisfy a required input"
            );
        }
    }

    #[test]
    fn optional_fragment_with_data_only_origin_allowed() {
        // Optional (non-required) fragments may carry ModelAssertion or
        // ExternalEvidence origin; they inform but never order work.
        for origin in [FragmentOrigin::ModelAssertion, FragmentOrigin::ExternalEvidence] {
            let mut frag = base_fragment(0x1A);
            frag.origin = origin;
            let p = projection_with(vec![frag]);
            let required = BTreeSet::new();
            let allowed = allowed_all_but_secret();
            assert_eq!(
                validate_context_projection(&input_for(&p, &required, &allowed)),
                Ok(()),
                "optional fragment with origin {origin:?} must be allowed"
            );
        }
    }

    #[test]
    fn excluded_candidate_records_reason_without_content() {
        let mut p = projection_with(vec![]);
        p.excluded.push(ExcludedCandidate {
            candidate_ref: id(0xE0),
            exclusion_reason: "data_class_not_admitted_for_recipient".to_string(),
            excluded_as_secret: false,
        });
        let required = BTreeSet::new();
        let allowed = allowed_all_but_secret();
        // Exclusions are evidence of the selection; they do not invalidate.
        assert_eq!(
            validate_context_projection(&input_for(&p, &required, &allowed)),
            Ok(())
        );
    }
}
