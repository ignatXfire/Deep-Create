use dc_domain::{ids::EntityId, scheduling::*};
use std::{
    cmp::Ordering,
    num::{NonZeroU16, NonZeroU32, NonZeroU64},
};

fn n(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}
fn w(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).unwrap()
}
fn constraints() -> SchedulingConstraints {
    SchedulingConstraints {
        max_weight: w(4),
        max_attempt_wall_time_ms: n(300_000),
        max_parallel_slots: NonZeroU32::new(4).unwrap(),
    }
}
fn proposal() -> SchedulingPolicy {
    SchedulingPolicy {
        policy_version: 1,
        effective_from_event: n(1),
        service_window_ms: n(300_000),
        root_aging_ms: n(60_000),
        unit_aging_ms: n(120_000),
        weights: PriorityWeights {
            foreground: w(4),
            deadline_critical: w(3),
            normal: w(2),
            background: w(1),
            exploration: w(1),
        },
        attempt_wall_time_cap_ms: n(300_000),
        protected_claims_max: 1,
        claim_drain_margin_ms: n(5_000),
    }
}
fn policy() -> ValidatedSchedulingPolicy {
    validate_scheduling_policy(&proposal(), constraints()).unwrap()
}
fn candidate(id: u128, priority: WorkPriority) -> RootServiceView {
    RootServiceView {
        root_work_id: EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | id).unwrap(),
        waiting_turn_seq: n(1),
        owner_priority: priority,
        completed_in_window_ms: 0,
        unfinished_conservative_ms: 0,
        candidate_parallel_slots: NonZeroU32::MIN,
        candidate_wall_time_ms: n(40_000),
    }
}

fn turn_candidate(
    id: u128,
    priority: WorkPriority,
    waiting_turn_seq: u64,
    aging_wait_started_ms: Option<u64>,
) -> RootTurnCandidate {
    RootTurnCandidate {
        view: RootServiceView {
            waiting_turn_seq: n(waiting_turn_seq),
            ..candidate(id, priority)
        },
        aging_wait_started_ms,
        active_protected_claim: false,
        skip_reason: None,
    }
}

#[test]
fn documented_ordinary_comparison_has_finite_foreground_advantage() {
    let a = candidate(1, WorkPriority::Foreground);
    let b = candidate(2, WorkPriority::Normal);
    let cmp = compare_root_service(a, b, &policy()).unwrap();
    assert_eq!(cmp.ordering, Ordering::Less);
    assert_eq!(cmp.reason, ComparisonReason::NormalizedService);
    assert_eq!(
        (cmp.left_numerator_ms, cmp.right_numerator_ms),
        (40_000, 40_000)
    );
    let busy_a = RootServiceView {
        unfinished_conservative_ms: 200_000,
        ..a
    };
    assert_eq!(
        compare_root_service(busy_a, b, &policy()).unwrap().ordering,
        Ordering::Greater
    );
}

#[test]
fn exact_fraction_then_ticket_then_identity_without_rounding() {
    let a = candidate(1, WorkPriority::Foreground);
    let b = RootServiceView {
        candidate_wall_time_ms: n(20_000),
        waiting_turn_seq: n(2),
        ..candidate(2, WorkPriority::Normal)
    };
    assert_eq!(
        compare_root_service(a, b, &policy()).unwrap().reason,
        ComparisonReason::WaitingTurn
    );
    let b = RootServiceView {
        waiting_turn_seq: n(1),
        ..b
    };
    assert_eq!(
        compare_root_service(a, b, &policy()).unwrap().reason,
        ComparisonReason::StableIdentity
    );
    let a = RootServiceView {
        completed_in_window_ms: (1_u64 << 53) + 1,
        candidate_wall_time_ms: n(1),
        ..a
    };
    let b = RootServiceView {
        completed_in_window_ms: 1_u64 << 53,
        root_work_id: EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0002).unwrap(),
        ..a
    };
    assert_eq!(
        compare_root_service(a, b, &policy()).unwrap().ordering,
        Ordering::Greater
    );
}

#[test]
fn invalid_policy_and_unenforceable_attempt_are_closed_errors() {
    for bad in [
        SchedulingPolicy {
            policy_version: 2,
            ..proposal()
        },
        SchedulingPolicy {
            protected_claims_max: 2,
            ..proposal()
        },
    ] {
        assert_eq!(
            validate_scheduling_policy(&bad, constraints()),
            Err(SchedulingError::InvalidPolicy)
        );
    }
    let mut bad = proposal();
    bad.weights.background = w(5);
    assert_eq!(
        validate_scheduling_policy(&bad, constraints()),
        Err(SchedulingError::InvalidPolicy)
    );
    bad = proposal();
    bad.attempt_wall_time_cap_ms = n(300_001);
    assert_eq!(
        validate_scheduling_policy(&bad, constraints()),
        Err(SchedulingError::UnboundedAttempt)
    );
    let a = candidate(1, WorkPriority::Normal);
    assert_eq!(
        compare_root_service(
            RootServiceView {
                candidate_wall_time_ms: n(300_001),
                ..a
            },
            a,
            &policy()
        ),
        Err(SchedulingError::UnboundedAttempt)
    );
    assert_eq!(
        compare_root_service(
            RootServiceView {
                candidate_parallel_slots: NonZeroU32::new(5).unwrap(),
                ..a
            },
            a,
            &policy()
        ),
        Err(SchedulingError::UnboundedAttempt)
    );
}

#[test]
fn all_additions_and_profile_products_reject_overflow() {
    let a = candidate(1, WorkPriority::Normal);
    assert_eq!(
        compare_root_service(
            RootServiceView {
                completed_in_window_ms: u64::MAX,
                ..a
            },
            a,
            &policy()
        ),
        Err(SchedulingError::ArithmeticOverflow)
    );
    let bad = SchedulingPolicy {
        attempt_wall_time_cap_ms: n(u64::MAX),
        ..proposal()
    };
    let bounds = SchedulingConstraints {
        max_attempt_wall_time_ms: n(u64::MAX),
        ..constraints()
    };
    assert_eq!(
        validate_scheduling_policy(&bad, bounds),
        Err(SchedulingError::ArithmeticOverflow)
    );
    let bad = SchedulingPolicy {
        attempt_wall_time_cap_ms: n(u64::MAX / 2),
        ..proposal()
    };
    assert_eq!(
        validate_scheduling_policy(&bad, bounds),
        Err(SchedulingError::ArithmeticOverflow)
    );
}

#[test]
fn comparison_is_antisymmetric_and_transitive_over_bounded_domain() {
    let policy = policy();
    let mut cases = Vec::new();
    for priority in [
        WorkPriority::Foreground,
        WorkPriority::Normal,
        WorkPriority::Exploration,
    ] {
        for service in [0, 20_000, 80_000] {
            for ticket in [1, 2] {
                cases.push(RootServiceView {
                    completed_in_window_ms: service,
                    waiting_turn_seq: n(ticket),
                    ..candidate(cases.len() as u128 + 1, priority)
                });
            }
        }
    }
    for &a in &cases {
        for &b in &cases {
            let ab = compare_root_service(a, b, &policy).unwrap().ordering;
            assert_eq!(
                ab.reverse(),
                compare_root_service(b, a, &policy).unwrap().ordering
            );
            for &c in &cases {
                if ab != Ordering::Greater
                    && compare_root_service(b, c, &policy).unwrap().ordering != Ordering::Greater
                {
                    assert_ne!(
                        compare_root_service(a, c, &policy).unwrap().ordering,
                        Ordering::Greater
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// choose_fair_root_turn tests (document 25 §4.3–4.5)
// ---------------------------------------------------------------------------

#[test]
fn ac_sch_01_reproducibility_no_float_no_hash_order() {
    // Same candidates, same policy, same now → identical decision twice.
    let policy = policy();
    let now = 1_000_000_u64;
    let candidates = vec![
        turn_candidate(1, WorkPriority::Foreground, 1, Some(now - 10_000)),
        turn_candidate(2, WorkPriority::Background, 2, Some(now - 70_000)),
        turn_candidate(3, WorkPriority::Normal, 3, Some(now - 5_000)),
    ];
    let d1 = choose_fair_root_turn(&candidates, &policy, now);
    let d2 = choose_fair_root_turn(&candidates, &policy, now);
    assert_eq!(d1, d2);
    // Also verify that the decision is not affected by hash-map iteration
    // order: we run it in a loop with the same inputs.
    for _ in 0..100 {
        assert_eq!(choose_fair_root_turn(&candidates, &policy, now), d1);
    }
}

#[test]
fn ac_sch_02_aged_background_wins_over_foreground() {
    // Foreground arrives continuously, background waits past root_aging_ms.
    let policy = policy();
    let now = 1_000_000_u64;
    let mut candidates = vec![turn_candidate(
        1,
        WorkPriority::Background,
        1,
        Some(now - 70_000),
    )];
    // Add many foreground candidates that are not yet aged.
    for i in 2..=10 {
        candidates.push(turn_candidate(
            i,
            WorkPriority::Foreground,
            i as u64,
            Some(now - 10_000),
        ));
    }
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 0);
            assert_eq!(reason, SelectionReason::AgingEldest);
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn ac_sch_02_aging_not_yet_eligible_falls_back_to_ordinary() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidates = vec![
        turn_candidate(1, WorkPriority::Background, 1, Some(now - 59_999)), // 1ms short
        turn_candidate(2, WorkPriority::Foreground, 2, Some(now)),
    ];
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 1); // foreground wins ordinary comparison
            assert_eq!(reason, SelectionReason::OrdinaryService);
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn ac_sch_02_protected_claim_short_circuits_ordinary() {
    let policy = policy();
    let now = 1_000_000_u64;
    let mut candidates = vec![
        turn_candidate(1, WorkPriority::Foreground, 1, Some(now - 10_000)),
        turn_candidate(2, WorkPriority::Background, 2, Some(now - 70_000)),
    ];
    // Give the background candidate an active protected claim.
    candidates[1].active_protected_claim = true;
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 1);
            assert_eq!(reason, SelectionReason::ProtectedClaim);
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn ac_sch_02_skip_reasons_exclude_candidates() {
    let policy = policy();
    let now = 1_000_000_u64;
    let mut c = turn_candidate(1, WorkPriority::Foreground, 1, Some(now - 70_000));
    c.skip_reason = Some(SkipReason::NoAdmissibleConnection);
    let candidates = vec![c];
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    assert_eq!(
        decision,
        SchedulingDecision::NoEligibleCandidates {
            reasons: vec![SkipReason::NoAdmissibleConnection],
        }
    );
}

#[test]
fn ac_sch_02_multiple_aged_eldest_wins() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidates = vec![
        turn_candidate(1, WorkPriority::Background, 5, Some(now - 100_000)),
        turn_candidate(2, WorkPriority::Background, 3, Some(now - 200_000)), // eldest
        turn_candidate(3, WorkPriority::Background, 7, Some(now - 80_000)),
    ];
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 1);
            assert_eq!(reason, SelectionReason::AgingEldest);
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn ac_sch_02_aging_tiebreak_by_root_work_id() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidates = vec![
        turn_candidate(2, WorkPriority::Background, 1, Some(now - 70_000)),
        turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000)),
    ];
    let decision = choose_fair_root_turn(&candidates, &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index, ..
        } => {
            assert_eq!(candidate_index, 1); // smaller root_work_id wins
        }
        _ => panic!("expected Selected"),
    }
}

// ---------------------------------------------------------------------------
// plan_protected_capacity_claim tests (document 25 §4.4)
// ---------------------------------------------------------------------------

fn entity_id(low: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | low).unwrap()
}

#[test]
fn ac_sch_03_multi_slot_claim_blocks_single_slot_takeovers() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidate = turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000));
    let limit_id = entity_id(100);
    let window_id = entity_id(200);
    let required = vec![RequiredSlot {
        limit_id,
        window_id,
        quantity: 2,
    }];
    let occupancy = vec![SlotOccupancy {
        limit_id,
        window_id,
        total_capacity: 2,
        occupied: 1,
        unconfirmed_release: 0,
    }];
    let proposal = plan_protected_capacity_claim(
        &candidate,
        &required,
        &occupancy,
        entity_id(300),
        1,
        42,
        entity_id(400),
        entity_id(500),
        entity_id(600),
        entity_id(700),
        now,
        &policy,
    )
    .unwrap();
    assert_eq!(proposal.state, ClaimState::Draining);
    assert_eq!(proposal.required_slot_vector.len(), 1);
    assert_eq!(proposal.required_slot_vector[0].quantity, 2);

    // A single-slot unrelated dispatch must NOT be allowed because it would
    // leave free capacity (2-1-1=0) below the claim requirement (2).
    let dispatch = vec![RequiredSlot {
        limit_id,
        window_id,
        quantity: 1,
    }];
    assert!(!residual_capacity_allows(
        &[proposal.clone()],
        &occupancy,
        &dispatch
    ));

    // After the claim is consumed, residual capacity is restored.
    let mut consumed = proposal;
    consumed.state = ClaimState::Consumed;
    assert!(residual_capacity_allows(&[consumed], &occupancy, &dispatch));
}

#[test]
fn ac_sch_05_upper_bound_arithmetic_rejects_overflow() {
    // Large U values that overflow u64 must be rejected by compare_root_service.
    let policy = policy();
    let a = candidate(1, WorkPriority::Normal);
    let b = RootServiceView {
        completed_in_window_ms: u64::MAX,
        ..a
    };
    assert_eq!(
        compare_root_service(b, a, &policy),
        Err(SchedulingError::ArithmeticOverflow)
    );
}

#[test]
fn ac_sch_09_unconfirmed_release_refuses_claim() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidate = turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000));
    let limit_id = entity_id(100);
    let window_id = entity_id(200);
    let required = vec![RequiredSlot {
        limit_id,
        window_id,
        quantity: 1,
    }];
    let occupancy = vec![SlotOccupancy {
        limit_id,
        window_id,
        total_capacity: 2,
        occupied: 1,
        unconfirmed_release: 1, // release not confirmed
    }];
    let result = plan_protected_capacity_claim(
        &candidate,
        &required,
        &occupancy,
        entity_id(300),
        1,
        42,
        entity_id(400),
        entity_id(500),
        entity_id(600),
        entity_id(700),
        now,
        &policy,
    );
    assert_eq!(result, Err(ClaimRefusal::CapacityReleaseUnconfirmed));
}

#[test]
fn ac_sch_09_capacity_busy_when_required_exceeds_total() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidate = turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000));
    let limit_id = entity_id(100);
    let window_id = entity_id(200);
    let required = vec![RequiredSlot {
        limit_id,
        window_id,
        quantity: 5, // more than total capacity
    }];
    let occupancy = vec![SlotOccupancy {
        limit_id,
        window_id,
        total_capacity: 2,
        occupied: 0,
        unconfirmed_release: 0,
    }];
    let result = plan_protected_capacity_claim(
        &candidate,
        &required,
        &occupancy,
        entity_id(300),
        1,
        42,
        entity_id(400),
        entity_id(500),
        entity_id(600),
        entity_id(700),
        now,
        &policy,
    );
    assert_eq!(result, Err(ClaimRefusal::CapacityBusy));
}

#[test]
fn ac_sch_13_followhead_instability_not_hidden() {
    // A FollowHead candidate whose dependency changed must not be silently
    // retried; it must be excluded with DependencyChanged, and after the
    // dependency stabilizes it must be eligible again.
    let policy = policy();
    let now = 1_000_000_u64;
    let mut unstable = turn_candidate(1, WorkPriority::Foreground, 1, Some(now - 70_000));
    unstable.skip_reason = Some(SkipReason::DependencyChanged);
    let stable = turn_candidate(2, WorkPriority::Background, 2, Some(now - 70_000));

    // While unstable, the stable candidate wins.
    let decision = choose_fair_root_turn(&[unstable, stable], &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 1);
            assert_eq!(reason, SelectionReason::AgingEldest);
        }
        _ => panic!("expected Selected"),
    }

    // After stabilization, the foreground candidate wins ordinary comparison.
    unstable.skip_reason = None;
    let decision = choose_fair_root_turn(&[unstable, stable], &policy, now);
    match decision {
        SchedulingDecision::Selected {
            candidate_index,
            reason,
            ..
        } => {
            assert_eq!(candidate_index, 0);
            assert_eq!(reason, SelectionReason::AgingEldest); // both aged; ticket 1 wins
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn residual_capacity_allows_independent_dispatch() {
    // An unrelated dispatch on a different slot group is allowed.
    let policy = policy();
    let now = 1_000_000_u64;
    let candidate = turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000));
    let limit_a = entity_id(100);
    let window_a = entity_id(200);
    let limit_b = entity_id(101);
    let window_b = entity_id(201);
    let required = vec![RequiredSlot {
        limit_id: limit_a,
        window_id: window_a,
        quantity: 1,
    }];
    let occupancy = vec![
        SlotOccupancy {
            limit_id: limit_a,
            window_id: window_a,
            total_capacity: 1,
            occupied: 0,
            unconfirmed_release: 0,
        },
        SlotOccupancy {
            limit_id: limit_b,
            window_id: window_b,
            total_capacity: 1,
            occupied: 0,
            unconfirmed_release: 0,
        },
    ];
    let proposal = plan_protected_capacity_claim(
        &candidate,
        &required,
        &occupancy,
        entity_id(300),
        1,
        42,
        entity_id(400),
        entity_id(500),
        entity_id(600),
        entity_id(700),
        now,
        &policy,
    )
    .unwrap();
    // Dispatch on limit_b is allowed (does not touch claim's group).
    let dispatch = vec![RequiredSlot {
        limit_id: limit_b,
        window_id: window_b,
        quantity: 1,
    }];
    assert!(residual_capacity_allows(&[proposal], &occupancy, &dispatch));
}
// Bug reproduction test: duplicate RequiredSlot entries over-claim capacity.
// Two entries of quantity=1 on same (limit_id, window_id) with total_capacity=1.
// Aggregated demand = 2 > total 1. Should be Err(CapacityBusy). Currently Ok.

#[test]
fn bug_duplicate_required_slot_over_claims() {
    let policy = policy();
    let now = 1_000_000_u64;
    let candidate = turn_candidate(1, WorkPriority::Background, 1, Some(now - 70_000));
    let limit_id = entity_id(100);
    let window_id = entity_id(200);
    // TWO identical entries — same slot group, each quantity=1
    let required = vec![
        RequiredSlot { limit_id, window_id, quantity: 1 },
        RequiredSlot { limit_id, window_id, quantity: 1 },
    ];
    let occupancy = vec![SlotOccupancy {
        limit_id,
        window_id,
        total_capacity: 1,
        occupied: 0,
        unconfirmed_release: 0,
    }];
    let result = plan_protected_capacity_claim(
        &candidate, &required, &occupancy,
        entity_id(300), 1, 42,
        entity_id(400), entity_id(500), entity_id(600), entity_id(700),
        now, &policy,
    );
    // Aggregated demand is 2 > total 1, must be refused
    assert_eq!(result, Err(ClaimRefusal::CapacityBusy),
        "duplicate RequiredSlot must aggregate: 2*1 > total 1");
}
