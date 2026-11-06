//! Synthetic C-01 facts, not provider billing or a database transaction.
use dc_domain::ids::{EntityId, RowVersion};
use dc_domain::resource::*;
use dc_domain::usage_binding::*;

fn id(n: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | n).unwrap()
}
fn reserved(n: u128, upper: UpperBound) -> LedgerPosition {
    let limit = ResourceLimit {
        limit_id: id(10 + n),
        scope: ResourceScope::RootWork(id(1)),
        window_id: id(20),
        kind: ResourceKind::Requests,
        bound: ExplicitLimit::Unlimited,
        opens_at_utc_millis: 0,
        closes_at_utc_millis: None,
        status: LimitStatus::Active,
    };
    evaluate_reservation(
        &limit,
        &[],
        &ReservationRequest {
            key: LedgerKey {
                logical_operation_id: id(30 + n),
                limit_id: limit.limit_id,
                window_id: limit.window_id,
            },
            reservation_id: id(40 + n),
            attempt_id: id(50),
            intent_id: id(60 + n),
            kind: limit.kind,
            upper_bound: upper,
        },
        1,
    )
    .unwrap()
    .position
}
fn observe(p: &LedgerPosition, n: u128, kind: ObservationKind) -> LedgerPosition {
    apply_observation(
        p,
        ResourceObservation {
            observation_id: id(n),
            reservation_id: p.request().reservation_id,
            ledger_key: p.request().key,
            resource_kind: p.request().kind,
            kind,
        },
    )
    .unwrap()
    .position
}
fn expected(p: &LedgerPosition) -> ExpectedUsagePosition {
    ExpectedUsagePosition {
        key: p.request().key,
        reservation_id: p.request().reservation_id,
        kind: p.request().kind,
        row_version: p.row_version(),
    }
}
fn settled(n: u128) -> LedgerPosition {
    observe(
        &reserved(n, UpperBound::Known(10)),
        100 + n,
        ObservationKind::Settled {
            confirmed_amount: 4,
        },
    )
}

#[test]
fn binding_is_repeatable_and_preserves_request_state_and_provenance() {
    let first = reserved(1, UpperBound::Known(10));
    let partial = observe(
        &first,
        100,
        ObservationKind::Partial {
            confirmed_amount: 2,
            unknown_remaining: UpperBound::Known(8),
        },
    );
    let position = observe(
        &partial,
        101,
        ObservationKind::Settled {
            confirmed_amount: 4,
        },
    );
    let before = position.clone();
    let a = bind_usage(
        &[expected(&position)],
        std::slice::from_ref(&position),
        id(50),
    )
    .unwrap();
    let b = bind_usage(
        &[expected(&position)],
        std::slice::from_ref(&position),
        id(50),
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(position, before);
    let bound = &a.positions()[0];
    assert_eq!(bound.request(), position.request());
    assert_eq!(
        bound.state(),
        LedgerState::Settled {
            confirmed_amount: 4
        }
    );
    assert_eq!(bound.contribution(), UpperBound::Known(4));
    assert_eq!(bound.observation_ids(), &[id(100), id(101)]);
    assert_eq!(bound.expected().row_version, position.row_version());
}

#[test]
fn unknown_accounting_is_preserved_and_does_not_release_its_remainder() {
    for upper in [UpperBound::Known(10), UpperBound::Unknown] {
        let position = observe(&reserved(1, upper), 100, ObservationKind::OutcomeUnknown);
        let binding = bind_usage(
            &[expected(&position)],
            std::slice::from_ref(&position),
            id(50),
        )
        .unwrap();
        assert_eq!(
            binding.positions()[0].state(),
            LedgerState::Unknown {
                confirmed_amount: 0,
                unknown_remaining: upper,
            }
        );
        assert_eq!(binding.positions()[0].contribution(), upper);
        assert_eq!(position.state(), binding.positions()[0].state());
    }
    let position = observe(
        &reserved(1, UpperBound::Known(10)),
        100,
        ObservationKind::Partial {
            confirmed_amount: 4,
            unknown_remaining: UpperBound::Known(6),
        },
    );
    let binding = bind_usage(&[expected(&position)], &[position], id(50)).unwrap();
    assert_eq!(binding.positions()[0].contribution(), UpperBound::Known(10));
}

#[test]
fn empty_unobserved_and_duplicate_rows_are_not_evidence() {
    assert_eq!(
        bind_usage(&[], &[], id(50)),
        Err(UsageBindingError::EmptyOrOversized)
    );
    let reserved = reserved(1, UpperBound::Known(10));
    assert_eq!(
        bind_usage(&[expected(&reserved)], &[reserved], id(50)),
        Err(UsageBindingError::NoObservation)
    );
    let a = settled(1);
    let b = settled(2);
    assert_eq!(
        bind_usage(
            &[expected(&a), expected(&a)],
            &[a.clone(), b.clone()],
            id(50)
        ),
        Err(UsageBindingError::DuplicateKey)
    );
    assert_eq!(
        bind_usage(&[expected(&a), expected(&b)], &[a.clone(), a], id(50)),
        Err(UsageBindingError::DuplicateKey)
    );
}

#[test]
fn exact_row_sets_are_required_but_input_order_is_not_significant() {
    let a = settled(1);
    let b = settled(2);
    let manifest = [expected(&a), expected(&b)];
    let ordered = bind_usage(&manifest, &[a.clone(), b.clone()], id(50)).unwrap();
    let reversed = bind_usage(&manifest, &[b.clone(), a.clone()], id(50)).unwrap();
    assert_eq!(ordered, reversed);
    assert_eq!(
        bind_usage(&manifest, std::slice::from_ref(&a), id(50)),
        Err(UsageBindingError::ManifestMismatch)
    );
    assert_eq!(
        bind_usage(&[expected(&a)], &[a, b], id(50)),
        Err(UsageBindingError::ManifestMismatch)
    );
}

#[test]
fn metadata_dimension_generation_and_attempt_cannot_be_substituted() {
    let position = settled(1);
    for case in 0..6 {
        let mut e = expected(&position);
        match case {
            0 => e.key.logical_operation_id = id(900),
            1 => e.key.limit_id = id(900),
            2 => e.key.window_id = id(900),
            3 => e.reservation_id = id(900),
            4 => e.kind = ResourceKind::MoneyMinorUnits { currency: id(900) },
            5 => e.row_version = RowVersion::new(900).unwrap(),
            _ => unreachable!(),
        }
        assert!(
            bind_usage(&[e], std::slice::from_ref(&position), id(50)).is_err(),
            "case {case}"
        );
    }
    assert_eq!(
        bind_usage(&[expected(&position)], &[position], id(51)),
        Err(UsageBindingError::AttemptMismatch)
    );
}

#[test]
fn late_bill_change_requires_new_basis_and_breach_is_not_hidden() {
    let position = settled(1);
    let manifest = [expected(&position)];
    let updated = observe(
        &position,
        102,
        ObservationKind::Settled {
            confirmed_amount: 5,
        },
    );
    assert_eq!(
        bind_usage(&manifest, std::slice::from_ref(&updated), id(50)),
        Err(UsageBindingError::PositionMismatch)
    );
    assert!(bind_usage(&[expected(&updated)], &[updated], id(50)).is_ok());
    let breached = observe(
        &position,
        103,
        ObservationKind::Settled {
            confirmed_amount: 11,
        },
    );
    assert!(breached.bound_breached());
    let binding = bind_usage(&[expected(&breached)], &[breached], id(50)).unwrap();
    assert!(binding.positions()[0].bound_breached());
    assert_eq!(binding.positions()[0].contribution(), UpperBound::Known(11));
    assert_eq!(
        position.state(),
        LedgerState::Settled {
            confirmed_amount: 4
        }
    );
}

#[test]
fn released_zero_requires_the_already_recorded_fact() {
    let position = observe(
        &reserved(1, UpperBound::Known(10)),
        100,
        ObservationKind::NotDispatchedConfirmed,
    );
    let binding = bind_usage(&[expected(&position)], &[position], id(50)).unwrap();
    assert_eq!(binding.positions()[0].state(), LedgerState::Released);
    assert_eq!(binding.positions()[0].contribution(), UpperBound::Known(0));
    assert_eq!(binding.positions()[0].observation_ids(), &[id(100)]);
}

#[test]
fn binding_respects_structural_input_and_observation_bounds() {
    let max = dc_domain::structure::STRUCTURAL_LIMITS_V1.max_collection_items;
    let position = settled(1);
    assert_eq!(
        bind_usage(
            &vec![expected(&position); max + 1],
            std::slice::from_ref(&position),
            id(50)
        ),
        Err(UsageBindingError::EmptyOrOversized)
    );
    let mut position = position;
    for n in 0..max {
        position = observe(
            &position,
            1000 + n as u128,
            ObservationKind::Settled {
                confirmed_amount: 4,
            },
        );
    }
    assert_eq!(
        bind_usage(&[expected(&position)], &[position], id(50)),
        Err(UsageBindingError::EmptyOrOversized)
    );
}
