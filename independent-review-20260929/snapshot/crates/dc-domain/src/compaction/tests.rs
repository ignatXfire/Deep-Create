//! Tests for doc 15 §5.3 verifiable compaction. All fixtures are synthetic.

use super::*;
use crate::context::SchemaVersion;
use crate::ids::{EntityId, RowVersion};


fn id(n: u128) -> EntityId {
    // UUIDv7-layout ids (version 7, variant 2) so EntityId accepts them.
    EntityId::from_u128(0x017f_22e2_79b0_7cc3_98c4_dc0c_0000_0000_u128 + (n << 8) + 0x07)
        .expect("fixture id")
}

fn rev(n: u64) -> RowVersion {
    RowVersion::new(n).expect("non-zero row version")
}

fn schema(n: u32) -> SchemaVersion {
    SchemaVersion::new(n).expect("non-zero schema version")
}

fn coverage_entry(local: &str, source: EntityId) -> CoverageEntry {
    CoverageEntry {
        record_local_id: local.to_string(),
        source_ref: source,
        origin: SummaryOrigin::Canonical,
        source_fingerprint: [7u8; 32],
        preserves_negation: true,
    }
}

fn base_candidate() -> SummaryCandidate {
    let s1 = id(0x100);
    let s2 = id(0x200);
    SummaryCandidate {
        candidate_id: id(0x900),
        objective_id: id(0x800),
        objective_revision: rev(3),
        source_range_start: s1,
        source_range_end: s2,
        carried_obligations: vec![id(0x0b11), id(0x0b12)],
        hypothesis_record_ids: vec!["rec-hyp".to_string()],
        retained_negative_result_ids: vec!["rec-neg".to_string()],
        right_states: vec![
            RightStateEntry {
                lease_id: id(0x1ea1),
                status: RightLifecycle::Live,
                observed_revision: rev(2),
            },
            RightStateEntry {
                lease_id: id(0x1ea2),
                status: RightLifecycle::Revoked,
                observed_revision: rev(5),
            },
        ],
        unknown_outcomes: vec![UnknownOutcomeEntry {
            outcome_ref: id(0x0e77),
            description: "external publish not yet observed".to_string(),
        }],
        coverage: vec![coverage_entry("rec-a", s1), coverage_entry("rec-b", s2)],
        transform_version: schema(1),
    }
}

fn input_for<'a>(
    candidate: &'a SummaryCandidate,
    open: &'a [EntityId],
    hyps: &'a [String],
    negs: &'a [String],
    rights: &'a [RightStateEntry],
    unknowns: &'a [EntityId],
    chain_ok: bool,
) -> SummaryVerificationInput<'a> {
    SummaryVerificationInput {
        candidate,
        open_or_violated_obligations: open,
        source_hypothesis_ids: hyps,
        source_negative_result_ids: negs,
        current_right_states: rights,
        source_unknown_outcomes: unknowns,
        prior_chain_reaches_canonical: chain_ok,
    }
}

#[test]
fn pass_when_everything_is_preserved() {
    let c = base_candidate();
    let open = vec![id(0x0b11), id(0x0b12)];
    let hyps = vec!["rec-hyp".to_string()];
    let negs = vec!["rec-neg".to_string()];
    let rights = vec![
        RightStateEntry {
            lease_id: id(0x1ea1),
            status: RightLifecycle::Live,
            observed_revision: rev(2),
        },
        RightStateEntry {
            lease_id: id(0x1ea2),
            status: RightLifecycle::Revoked,
            observed_revision: rev(5),
        },
    ];
    let unknowns = vec![id(0x0e77)];
    let input = input_for(&c, &open, &hyps, &negs, &rights, &unknowns, true);
    assert_eq!(verify_summary_candidate(&input).unwrap(), Verdict::Pass);
    assert!(may_serve_as_authority(Verdict::Pass));
    assert!(!may_serve_as_authority(Verdict::Fail));
}

#[test]
fn empty_coverage_is_rejected() {
    let mut c = base_candidate();
    c.coverage.clear();
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &[], &[], &[], &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::EmptyCoverage
    );
}

#[test]
fn lost_obligation_is_detected() {
    let mut c = base_candidate();
    c.carried_obligations.retain(|o| *o != id(0x0b12));
    let open = vec![id(0x0b11), id(0x0b12)];
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &open, &[], &[], &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::LostObligation(id(0x0b12))
    );
}

#[test]
fn hypothesis_promoted_to_fact_is_detected() {
    let mut c = base_candidate();
    c.hypothesis_record_ids.clear();
    let hyps = vec!["rec-hyp".to_string()];
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &[], &hyps, &[], &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::HypothesisPromoted("rec-hyp".to_string())
    );
}

#[test]
fn negative_result_lost_is_detected() {
    let mut c = base_candidate();
    c.retained_negative_result_ids.clear();
    let negs = vec!["rec-neg".to_string()];
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &[], &[], &negs, &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::NegativeResultLost("rec-neg".to_string())
    );
}

#[test]
fn revived_right_is_detected() {
    // Candidate claims the lease is Active...
    let mut c = base_candidate();
    c.right_states = vec![RightStateEntry {
        lease_id: id(0x1ea2),
        status: RightLifecycle::Live,
        observed_revision: rev(5),
    }];
    // ...but the current truth is Revoked.
    let rights = vec![RightStateEntry {
        lease_id: id(0x1ea2),
        status: RightLifecycle::Revoked,
        observed_revision: rev(6),
    }];
    let input = input_for(&c, &[], &[], &[], &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::RightRevived(id(0x1ea2))
    );
}

#[test]
fn expired_right_listed_as_expired_passes() {
    let mut c = base_candidate();
    c.right_states = vec![RightStateEntry {
        lease_id: id(0x1ea3),
        status: RightLifecycle::Expired,
        observed_revision: rev(4),
    }];
    let rights = vec![RightStateEntry {
        lease_id: id(0x1ea3),
        status: RightLifecycle::Expired,
        observed_revision: rev(4),
    }];
    let input = input_for(&c, &[], &[], &[], &rights, &[], true);
    assert_eq!(verify_summary_candidate(&input).unwrap(), Verdict::Pass);
}

#[test]
fn unknown_outcome_turned_into_failure_is_detected() {
    let mut c = base_candidate();
    c.unknown_outcomes.clear(); // silently resolved as failure
    let unknowns = vec![id(0x0e77)];
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &[], &[], &[], &rights, &unknowns, true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::UnknownOutcomeAsFailure(id(0x0e77))
    );
}

#[test]
fn prior_summary_chain_must_reach_canonical() {
    let mut c = base_candidate();
    c.coverage[1].origin = SummaryOrigin::PriorVerifiedSummary;
    let rights: Vec<RightStateEntry> = vec![];
    // Chain broken somewhere upstream.
    let input = input_for(&c, &[], &[], &[], &rights, &[], false);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::UntraceableToCanonicalSource("rec-b".to_string())
    );
    // Chain verified to reach canonical: passes.
    let input = input_for(&c, &[], &[], &[], &rights, &[], true);
    assert_eq!(verify_summary_candidate(&input).unwrap(), Verdict::Pass);
}

#[test]
fn oversized_collections_are_rejected() {
    let mut c = base_candidate();
    let limit = crate::structure::STRUCTURAL_LIMITS_V1.max_collection_items;
    c.coverage = (0..=limit)
        .map(|i| coverage_entry(&format!("rec-{i}"), id(0x100)))
        .collect();
    let rights: Vec<RightStateEntry> = vec![];
    let input = input_for(&c, &[], &[], &[], &rights, &[], true);
    assert_eq!(
        verify_summary_candidate(&input).unwrap_err(),
        SummaryError::Oversized
    );
}

#[test]
fn satisfied_obligation_need_not_be_carried() {
    // The open/violated list is the caller's ground truth; an obligation not
    // in it (e.g. Satisfied/Waived) is not required in carried_obligations.
    let c = base_candidate();
    let open = vec![id(0x0b11), id(0x0b12)]; // both carried in base_candidate
    let hyps = vec!["rec-hyp".to_string()];
    let negs = vec!["rec-neg".to_string()];
    let rights: Vec<RightStateEntry> = vec![];
    let unknowns = vec![id(0x0e77)];
    let input = input_for(&c, &open, &hyps, &negs, &rights, &unknowns, true);
    assert_eq!(verify_summary_candidate(&input).unwrap(), Verdict::Pass);
}
