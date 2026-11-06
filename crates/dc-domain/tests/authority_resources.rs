//! Synthetic C-01 tests only. These do not exercise Windows, persistence, or a
//! held AdmissionFence and must not be reported as process-isolation evidence.

use std::collections::{BTreeMap, BTreeSet};

use dc_domain::ids::{EntityId, RowVersion};
use dc_domain::permission::*;
use dc_domain::resource::*;

fn id(value: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | value)
        .expect("synthetic UUIDv7 fixture")
}
fn version(value: u64) -> RowVersion {
    RowVersion::new(value).expect("nonzero fixture")
}

fn lease(work: u128, identity: u128) -> CapabilityLease {
    CapabilityLease {
        lease_id: id(identity),
        owner_event_id: id(90),
        installation_id: id(1),
        scope: LeaseScope {
            work_id: id(work),
            unit_id: None,
            action_kind: ActionKind::CallModelConnection,
            resource: CanonicalResource::ModelConnection {
                connection_id: id(8),
                revision: version(1),
            },
            recipient: Some(CanonicalRecipient::ModelConnection {
                connection_id: id(8),
                revision: version(1),
            }),
            data_classes: BTreeSet::from([DataClass::Public, DataClass::Personal]),
            max_calls: ExplicitLimit::Finite(10),
            max_bytes: ExplicitLimit::Finite(1000),
            cost_limits: BTreeMap::from([(ResourceKind::Requests, ExplicitLimit::Finite(10))]),
        },
        not_before_utc_millis: 100,
        expiry: LeaseExpiry::At {
            utc_millis: 1000,
            monotonic: MonotonicDeadline {
                run_generation: version(1),
                at_millis: 800,
            },
        },
        parent_lease_id: None,
        delegation_depth: 0,
        max_delegation_depth: 2,
        authority_epoch: version(1),
        run_generation: version(1),
        support_fences: vec![
            ScopedFenceVersion {
                scope: FenceScope::RootWork(id(work)),
                version: version(1),
            },
            ScopedFenceVersion {
                scope: FenceScope::Lease(id(identity)),
                version: version(1),
            },
        ],
        confirmed_conditions_hash: [7; 32],
        issued_dependency_snapshot_id: id(91),
        lease_version: version(1),
        status: LeaseStatus::Active,
    }
}

fn child(parent: &CapabilityLease, identity: u128) -> CapabilityLease {
    let mut child = parent.clone();
    child.lease_id = id(identity);
    child.parent_lease_id = Some(parent.lease_id);
    child.delegation_depth = parent.delegation_depth + 1;
    child.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::Lease(id(identity)),
        version: version(1),
    });
    child
}

fn snapshot(leases: &[&CapabilityLease]) -> AuthoritySnapshot {
    AuthoritySnapshot {
        installation_id: id(1),
        authority_epoch: version(1),
        run_generation: version(1),
        fences: leases
            .iter()
            .flat_map(|lease| lease.support_fences.iter())
            .map(|fence| {
                (
                    fence.scope,
                    CurrentFence {
                        version: fence.version,
                        closed: false,
                    },
                )
            })
            .collect(),
    }
}

fn now() -> ClockObservation {
    ClockObservation {
        utc_millis: 500,
        monotonic_millis: 400,
        run_generation: version(1),
        integrity: ClockIntegrity::Trusted,
    }
}

#[test]
fn child_is_a_true_subset_in_each_limit_dimension() {
    let parent = lease(10, 20);
    for calls in 0..=11 {
        let mut candidate = child(&parent, 21);
        candidate.scope.max_calls = ExplicitLimit::Finite(calls);
        assert_eq!(is_lease_subset(&parent, &candidate).is_ok(), calls <= 10);
    }
    let mut candidate = child(&parent, 21);
    candidate.scope.unit_id = Some(id(30));
    candidate.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::Unit(id(30)),
        version: version(1),
    });
    candidate.scope.data_classes = BTreeSet::from([DataClass::Public]);
    candidate.scope.max_bytes = ExplicitLimit::Finite(1);
    assert_eq!(is_lease_subset(&parent, &candidate), Ok(()));
    candidate.scope.max_bytes = ExplicitLimit::Unlimited;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::LimitExpansion)
    );
    candidate.scope.max_bytes = ExplicitLimit::Finite(1);
    candidate.scope.cost_limits.clear();
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::LimitExpansion)
    );
}

#[test]
fn work_action_recipient_data_and_object_expansions_are_denied() {
    let parent = lease(10, 20);
    let mut candidate = child(&parent, 21);
    candidate.scope.work_id = id(11);
    candidate.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::RootWork(id(11)),
        version: version(1),
    });
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ScopeExpansion)
    );
    let mut candidate = child(&parent, 21);
    candidate.scope.resource = CanonicalResource::ModelConnection {
        connection_id: id(9),
        revision: version(1),
    };
    candidate.scope.recipient = Some(CanonicalRecipient::ModelConnection {
        connection_id: id(9),
        revision: version(1),
    });
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ScopeExpansion)
    );
    candidate = child(&parent, 21);
    candidate.scope.recipient = None;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ActionMismatch)
    );
    candidate = child(&parent, 21);
    candidate.scope.action_kind = ActionKind::CallRegisteredEndpoint;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ActionMismatch)
    );
    candidate = child(&parent, 21);
    candidate.scope.data_classes.insert(DataClass::Restricted);
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ScopeExpansion)
    );
}

#[test]
fn file_object_and_workspace_binding_are_exact_not_path_prefixes() {
    let mut parent = lease(10, 20);
    parent.scope.action_kind = ActionKind::ReadWorkspaceFile;
    parent.scope.resource = CanonicalResource::WorkspaceFile {
        workspace_id: id(70),
        binding_revision: version(1),
        object_id: id(71),
    };
    parent.scope.recipient = None;
    let candidate = child(&parent, 21);
    assert_eq!(is_lease_subset(&parent, &candidate), Ok(()));
    for resource in [
        CanonicalResource::WorkspaceFile {
            workspace_id: id(70),
            binding_revision: version(2),
            object_id: id(71),
        },
        CanonicalResource::WorkspaceFile {
            workspace_id: id(70),
            binding_revision: version(1),
            object_id: id(72),
        },
        CanonicalResource::WorkspaceFile {
            workspace_id: id(73),
            binding_revision: version(1),
            object_id: id(71),
        },
    ] {
        let mut expanded = candidate.clone();
        expanded.scope.resource = resource;
        assert_eq!(
            is_lease_subset(&parent, &expanded),
            Err(PermissionError::ScopeExpansion)
        );
    }
}

#[test]
fn unit_restriction_cannot_be_removed_or_transferred() {
    let mut parent = lease(10, 20);
    parent.scope.unit_id = Some(id(30));
    parent.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::Unit(id(30)),
        version: version(1),
    });
    let mut candidate = child(&parent, 21);
    candidate.scope.unit_id = None;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ScopeExpansion)
    );
    candidate.scope.unit_id = Some(id(31));
    candidate.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::Unit(id(31)),
        version: version(1),
    });
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ScopeExpansion)
    );
}

#[test]
fn child_cannot_extend_either_clock_or_remove_expiry() {
    let parent = lease(10, 20);
    for expiry in [
        LeaseExpiry::Never,
        LeaseExpiry::At {
            utc_millis: 1001,
            monotonic: MonotonicDeadline {
                run_generation: version(1),
                at_millis: 800,
            },
        },
        LeaseExpiry::At {
            utc_millis: 1000,
            monotonic: MonotonicDeadline {
                run_generation: version(1),
                at_millis: 801,
            },
        },
    ] {
        let mut candidate = child(&parent, 21);
        candidate.expiry = expiry;
        assert_eq!(
            is_lease_subset(&parent, &candidate),
            Err(PermissionError::ExpiryExpansion)
        );
    }
    let mut candidate = child(&parent, 21);
    candidate.not_before_utc_millis = 99;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::ExpiryExpansion)
    );
}

#[test]
fn delegation_depth_and_parent_support_cannot_be_bypassed() {
    let parent = lease(10, 20);
    let candidate = child(&parent, 21);
    let grandchild = child(&candidate, 22);
    assert_eq!(is_lease_subset(&candidate, &grandchild), Ok(()));
    let too_deep = child(&grandchild, 23);
    assert_eq!(
        is_lease_subset(&grandchild, &too_deep),
        Err(PermissionError::InvalidDelegation)
    );
    let mut candidate = child(&parent, 21);
    candidate
        .support_fences
        .retain(|fence| fence.scope != FenceScope::Lease(parent.lease_id));
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::InvalidSupportFences)
    );
    candidate = child(&parent, 21);
    candidate.max_delegation_depth = 3;
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::InvalidDelegation)
    );
}

#[test]
fn every_authority_support_is_checked_and_unrelated_root_survives_scoped_revoke() {
    let a = lease(10, 20);
    let b = lease(11, 21);
    let mut supported = child(&a, 22);
    supported.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::RootWork(id(12)),
        version: version(1),
    });
    let before = snapshot(&[&a, &b, &supported]);
    assert_eq!(verify_lease(&supported, version(1), &before, now()), Ok(()));
    for scope in [
        FenceScope::RootWork(id(10)),
        FenceScope::RootWork(id(12)),
        FenceScope::Lease(id(20)),
        FenceScope::Lease(id(22)),
    ] {
        let mut after = before.clone();
        after.fences.insert(
            scope,
            CurrentFence {
                version: version(2),
                closed: true,
            },
        );
        assert_eq!(
            verify_lease(&supported, version(1), &after, now()),
            Err(PermissionError::FenceChanged)
        );
        assert_eq!(verify_lease(&b, version(1), &after, now()), Ok(()));
        assert_eq!(after.authority_epoch, before.authority_epoch);
    }
    let mut changed = before.clone();
    changed
        .fences
        .get_mut(&FenceScope::RootWork(id(12)))
        .expect("support fixture")
        .version = version(2);
    assert_eq!(
        verify_lease(&supported, version(1), &changed, now()),
        Err(PermissionError::FenceChanged)
    );
}

#[test]
fn child_preserves_multi_support_and_duplicate_fences_fail_closed() {
    let mut parent = lease(10, 20);
    parent.support_fences.push(ScopedFenceVersion {
        scope: FenceScope::RootWork(id(12)),
        version: version(1),
    });
    let mut candidate = child(&parent, 21);
    candidate
        .support_fences
        .retain(|f| f.scope != FenceScope::RootWork(id(12)));
    assert_eq!(
        is_lease_subset(&parent, &candidate),
        Err(PermissionError::InvalidSupportFences)
    );
    let mut duplicate = parent.clone();
    duplicate.support_fences.push(parent.support_fences[0]);
    assert_eq!(
        verify_lease(&duplicate, version(1), &snapshot(&[&parent]), now()),
        Err(PermissionError::InvalidSupportFences)
    );
}

#[test]
fn expiry_is_rechecked_at_the_action_boundary_and_monotonic_time_prevents_revival() {
    let lease = lease(10, 20);
    let authority = snapshot(&[&lease]);
    assert_eq!(verify_lease(&lease, version(1), &authority, now()), Ok(()));
    let at_utc_expiry = ClockObservation {
        utc_millis: 1000,
        ..now()
    };
    assert_eq!(
        verify_lease(&lease, version(1), &authority, at_utc_expiry),
        Err(PermissionError::Expired)
    );
    let at_monotonic_expiry = ClockObservation {
        utc_millis: 999,
        monotonic_millis: 800,
        ..now()
    };
    assert_eq!(
        verify_lease(&lease, version(1), &authority, at_monotonic_expiry),
        Err(PermissionError::Expired)
    );
    for integrity in [ClockIntegrity::RolledBack, ClockIntegrity::Uncertain] {
        assert_eq!(
            verify_lease(
                &lease,
                version(1),
                &authority,
                ClockObservation { integrity, ..now() }
            ),
            Err(PermissionError::ClockUntrusted)
        );
    }
    assert_eq!(
        verify_lease(
            &lease,
            version(1),
            &authority,
            ClockObservation {
                utc_millis: 99,
                ..now()
            }
        ),
        Err(PermissionError::NotYetValid)
    );
}

#[test]
fn global_epoch_generation_and_lease_version_require_exact_equality() {
    let lease = lease(10, 20);
    let mut authority = snapshot(&[&lease]);
    authority.authority_epoch = version(2);
    assert_eq!(
        verify_lease(&lease, version(1), &authority, now()),
        Err(PermissionError::EpochMismatch)
    );
    authority.authority_epoch = version(1);
    authority.run_generation = version(2);
    assert_eq!(
        verify_lease(&lease, version(1), &authority, now()),
        Err(PermissionError::StaleGeneration)
    );
    authority.run_generation = version(1);
    assert_eq!(
        verify_lease(&lease, version(2), &authority, now()),
        Err(PermissionError::VersionChanged)
    );
    assert_eq!(
        next_fence_version(version(u64::MAX)),
        Err(PermissionError::ArithmeticOverflow)
    );
}

#[test]
fn no_terminal_lease_can_be_revived_or_relabelled() {
    for terminal in [
        LeaseStatus::Consumed,
        LeaseStatus::Expired,
        LeaseStatus::Revoked,
        LeaseStatus::Superseded,
    ] {
        assert_eq!(
            validate_lease_transition(LeaseStatus::Active, terminal),
            Ok(())
        );
        assert_eq!(
            validate_lease_transition(terminal, LeaseStatus::Active),
            Err(PermissionError::LeaseRevival)
        );
        let mut lease = lease(10, 20);
        lease.status = terminal;
        assert_eq!(
            verify_lease(&lease, version(1), &snapshot(&[&lease]), now()),
            Err(PermissionError::LeaseInactive)
        );
    }
    assert!(serde_json::from_str::<ActionKind>("\"PayAnything\"").is_err());
    assert!(serde_json::from_str::<ExplicitLimit<u64>>("null").is_err());
}

fn limit(identity: u128, bound: ExplicitLimit<u64>) -> ResourceLimit {
    ResourceLimit {
        limit_id: id(identity),
        scope: ResourceScope::RootWork(id(10)),
        window_id: id(50),
        kind: ResourceKind::Requests,
        bound,
        opens_at_utc_millis: 0,
        closes_at_utc_millis: Some(1000),
        status: LimitStatus::Active,
    }
}

fn request(limit: &ResourceLimit, operation: u128, upper: UpperBound) -> ReservationRequest {
    ReservationRequest {
        key: LedgerKey {
            logical_operation_id: id(operation),
            limit_id: limit.limit_id,
            window_id: limit.window_id,
        },
        reservation_id: id(operation + 1000),
        attempt_id: id(operation + 2000),
        intent_id: id(operation + 3000),
        kind: limit.kind,
        upper_bound: upper,
    }
}

fn reserve(limit: &ResourceLimit, operation: u128, upper: u64) -> LedgerPosition {
    evaluate_reservation(
        limit,
        &[],
        &request(limit, operation, UpperBound::Known(upper)),
        500,
    )
    .expect("valid synthetic reservation")
    .position
}

fn observe(
    position: &LedgerPosition,
    observation_id: u128,
    kind: ObservationKind,
) -> Result<ObservationAssessment, ResourceError> {
    apply_observation(
        position,
        ResourceObservation {
            observation_id: id(observation_id),
            reservation_id: position.request().reservation_id,
            ledger_key: position.request().key,
            resource_kind: position.request().kind,
            kind,
        },
    )
}

#[test]
fn reservation_boundaries_are_exact_and_missing_bounds_are_not_unlimited() {
    for max in 0..12 {
        let cap = limit(40, ExplicitLimit::Finite(max));
        for upper in 0..13 {
            let result = evaluate_reservation(
                &cap,
                &[],
                &request(&cap, 100, UpperBound::Known(upper)),
                500,
            );
            assert_eq!(result.is_ok(), upper <= max);
        }
        assert_eq!(
            evaluate_reservation(&cap, &[], &request(&cap, 100, UpperBound::Unknown), 500),
            Err(ResourceError::UnknownUpperBound)
        );
    }
    let cap = limit(40, ExplicitLimit::Unlimited);
    let assessment = evaluate_reservation(&cap, &[], &request(&cap, 100, UpperBound::Unknown), 500)
        .expect("explicit unlimited");
    assert_eq!(assessment.total_after, UpperBound::Unknown);
    let unknown = observe(&assessment.position, 200, ObservationKind::OutcomeUnknown)
        .expect("unknown")
        .position;
    assert_eq!(unknown.contribution(), Ok(UpperBound::Unknown));
}

#[test]
fn reserved_unknown_partial_and_settled_replace_one_contribution() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&cap, 100, 80);
    assert_eq!(reserved.contribution(), Ok(UpperBound::Known(80)));
    let unknown = observe(&reserved, 200, ObservationKind::OutcomeUnknown)
        .expect("unknown")
        .position;
    assert_eq!(unknown.contribution(), Ok(UpperBound::Known(80)));
    let partial = observe(
        &unknown,
        201,
        ObservationKind::Partial {
            confirmed_amount: 20,
            unknown_remaining: UpperBound::Known(60),
        },
    )
    .expect("partial")
    .position;
    assert_eq!(partial.contribution(), Ok(UpperBound::Known(80)));
    assert_eq!(
        evaluate_reservation(
            &cap,
            std::slice::from_ref(&partial),
            &request(&cap, 101, UpperBound::Known(21)),
            500
        ),
        Err(ResourceError::LimitReached)
    );
    let settled = observe(
        &partial,
        202,
        ObservationKind::Settled {
            confirmed_amount: 30,
        },
    )
    .expect("settled")
    .position;
    assert_eq!(settled.contribution(), Ok(UpperBound::Known(30)));
    let assessment = evaluate_reservation(
        &cap,
        &[settled],
        &request(&cap, 101, UpperBound::Known(70)),
        500,
    )
    .expect("only unused difference released");
    assert_eq!(assessment.total_after, UpperBound::Known(100));
}

#[test]
fn observation_idempotency_keeps_version_and_conflicting_replays_fail() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&cap, 100, 80);
    let kind = ObservationKind::Settled {
        confirmed_amount: 25,
    };
    let first = observe(&reserved, 200, kind).expect("first");
    let duplicate = observe(&first.position, 200, kind).expect("same observation");
    assert!(duplicate.already_applied);
    assert_eq!(first.position, duplicate.position);
    assert_eq!(
        observe(
            &first.position,
            200,
            ObservationKind::Settled {
                confirmed_amount: 26
            }
        ),
        Err(ResourceError::ReplayConflict)
    );
    let independent_same_total =
        observe(&first.position, 201, kind).expect("new cumulative observation");
    assert_eq!(
        independent_same_total.position.contribution(),
        Ok(UpperBound::Known(25))
    );
}

#[test]
fn late_cost_above_upper_is_retained_and_closes_subsequent_spending() {
    let cap = limit(40, ExplicitLimit::Finite(1000));
    let reserved = reserve(&cap, 100, 80);
    let unknown = observe(&reserved, 200, ObservationKind::OutcomeUnknown)
        .expect("unknown")
        .position;
    let late = observe(
        &unknown,
        201,
        ObservationKind::Settled {
            confirmed_amount: 200,
        },
    )
    .expect("late fact is not refused")
    .position;
    assert_eq!(late.contribution(), Ok(UpperBound::Known(200)));
    assert!(late.bound_breached());
    assert_eq!(
        evaluate_reservation(
            &cap,
            std::slice::from_ref(&late),
            &request(&cap, 101, UpperBound::Known(1)),
            500
        ),
        Err(ResourceError::BoundBreached)
    );
    let corrected = observe(
        &late,
        202,
        ObservationKind::Settled {
            confirmed_amount: 210,
        },
    )
    .expect("late correction retained")
    .position;
    assert_eq!(corrected.contribution(), Ok(UpperBound::Known(210)));
    assert!(corrected.bound_breached());
}

#[test]
fn duplicate_reservation_is_not_a_second_debit_or_another_intent() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&cap, 100, 80);
    let replay = evaluate_reservation(
        &cap,
        std::slice::from_ref(&reserved),
        reserved.request(),
        500,
    )
    .expect("same position");
    assert!(replay.already_exists);
    assert_eq!(replay.total_after, UpperBound::Known(80));
    let mut conflict = reserved.request().clone();
    conflict.intent_id = id(999);
    assert_eq!(
        evaluate_reservation(&cap, std::slice::from_ref(&reserved), &conflict, 500),
        Err(ResourceError::ReplayConflict)
    );
    conflict = reserved.request().clone();
    conflict.key.logical_operation_id = id(101);
    assert_eq!(
        evaluate_reservation(&cap, std::slice::from_ref(&reserved), &conflict, 500),
        Err(ResourceError::ReplayConflict)
    );
    assert_eq!(
        evaluate_reservation(
            &cap,
            &[reserved.clone(), reserved.clone()],
            reserved.request(),
            500
        ),
        Err(ResourceError::DuplicateLedgerKey)
    );
}

#[test]
fn unknown_old_attempt_remains_occupied_when_new_attempt_reserves() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let first = reserve(&cap, 100, 60);
    let unknown = observe(&first, 200, ObservationKind::OutcomeUnknown)
        .expect("unknown")
        .position;
    assert_eq!(
        evaluate_reservation(
            &cap,
            &[unknown],
            &request(&cap, 101, UpperBound::Known(60)),
            500
        ),
        Err(ResourceError::LimitReached)
    );
}

#[test]
fn children_share_parent_budget_and_cannot_understate_parent_charge() {
    let parent = limit(40, ExplicitLimit::Finite(100));
    let mut a = limit(41, ExplicitLimit::Finite(100));
    a.scope = ResourceScope::Branch(id(11));
    let mut b = limit(42, ExplicitLimit::Finite(100));
    b.scope = ResourceScope::Branch(id(12));
    let first_requests = [
        request(&parent, 100, UpperBound::Known(60)),
        request(&a, 100, UpperBound::Known(60)),
    ];
    let first = evaluate_reservations(&[parent.clone(), a], &[], &first_requests, 500)
        .expect("one operation in parent and child windows");
    let current: Vec<_> = first
        .into_iter()
        .map(|assessment| assessment.position)
        .collect();
    let second_requests = [
        request(&parent, 101, UpperBound::Known(60)),
        request(&b, 101, UpperBound::Known(60)),
    ];
    assert_eq!(
        evaluate_reservations(
            &[parent.clone(), b.clone()],
            &current,
            &second_requests,
            500
        ),
        Err(ResourceError::LimitReached)
    );
    let undercounted = [
        request(&parent, 101, UpperBound::Known(1)),
        request(&b, 101, UpperBound::Known(60)),
    ];
    assert_eq!(
        evaluate_reservations(&[parent.clone(), b.clone()], &current, &undercounted, 500),
        Err(ResourceError::LimitMismatch)
    );
    assert_eq!(
        evaluate_reservations(&[parent, b], &current, &second_requests[..1], 500),
        Err(ResourceError::MissingApplicableLimit)
    );
}

#[test]
fn reusable_slots_need_confirmed_termination_and_never_release_money() {
    let money = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&money, 100, 80);
    let unknown = observe(&reserved, 200, ObservationKind::OutcomeUnknown)
        .expect("unknown")
        .position;
    assert_eq!(
        observe(&unknown, 201, ObservationKind::ProcessTerminationConfirmed),
        Err(ResourceError::InvalidObservationTransition)
    );
    let mut slots = limit(41, ExplicitLimit::Finite(2));
    slots.kind = ResourceKind::ConcurrentSlots;
    let occupied = reserve(&slots, 101, 2);
    let unknown = observe(&occupied, 203, ObservationKind::OutcomeUnknown)
        .expect("unknown capacity")
        .position;
    assert_eq!(unknown.contribution(), Ok(UpperBound::Known(2)));
    let released = observe(&unknown, 204, ObservationKind::ProcessTerminationConfirmed)
        .expect("confirmed process end")
        .position;
    assert_eq!(released.contribution(), Ok(UpperBound::Known(0)));
    let separate_observation =
        observe(&released, 205, ObservationKind::ProcessTerminationConfirmed)
            .expect("new independent observation keeps the same zero contribution");
    assert!(!separate_observation.already_applied);
    assert_eq!(
        separate_observation.position.contribution(),
        Ok(UpperBound::Known(0))
    );
    assert_eq!(
        separate_observation.position.row_version().get(),
        released.row_version().get() + 1
    );
    let replay = observe(
        &separate_observation.position,
        205,
        ObservationKind::ProcessTerminationConfirmed,
    )
    .expect("same observation is idempotent");
    assert!(replay.already_applied);
    assert_eq!(replay.position, separate_observation.position);
}

#[test]
fn trusted_reconciliation_releases_unknown_zero_but_not_confirmed_spending() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&cap, 100, 80);
    let unknown = observe(&reserved, 200, ObservationKind::OutcomeUnknown)
        .expect("loss itself retains U")
        .position;
    assert_eq!(unknown.contribution(), Ok(UpperBound::Known(80)));
    let reconciled = observe(&unknown, 201, ObservationKind::NotDispatchedConfirmed)
        .expect("trusted broker proves D never occurred")
        .position;
    assert_eq!(reconciled.state(), LedgerState::Released);
    assert_eq!(reconciled.contribution(), Ok(UpperBound::Known(0)));
    let partial = observe(
        &unknown,
        202,
        ObservationKind::Partial {
            confirmed_amount: 1,
            unknown_remaining: UpperBound::Known(79),
        },
    )
    .expect("already observed cost")
    .position;
    assert_eq!(
        observe(&partial, 203, ObservationKind::NotDispatchedConfirmed),
        Err(ResourceError::InvalidObservationTransition)
    );
    assert_eq!(partial.contribution(), Ok(UpperBound::Known(80)));
}

#[test]
fn released_position_still_accepts_an_unexpected_late_bill_as_a_breach() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let reserved = reserve(&cap, 100, 80);
    let released = observe(&reserved, 200, ObservationKind::NotDispatchedConfirmed)
        .expect("trusted no-dispatch observation")
        .position;
    let late = observe(
        &released,
        201,
        ObservationKind::Settled {
            confirmed_amount: 10,
        },
    )
    .expect("conflicting late bill must remain visible")
    .position;
    assert_eq!(late.contribution(), Ok(UpperBound::Known(10)));
    assert!(late.bound_breached());
}

#[test]
fn integer_overflow_blocks_reservation_but_does_not_erase_observed_facts() {
    let cap = limit(40, ExplicitLimit::Unlimited);
    let reserved = reserve(&cap, 100, u64::MAX);
    assert_eq!(
        evaluate_reservation(
            &cap,
            std::slice::from_ref(&reserved),
            &request(&cap, 101, UpperBound::Known(1)),
            500
        ),
        Err(ResourceError::ArithmeticOverflow)
    );
    let observed = observe(
        &reserved,
        200,
        ObservationKind::Partial {
            confirmed_amount: u64::MAX,
            unknown_remaining: UpperBound::Known(1),
        },
    )
    .expect("retain exact C and R separately")
    .position;
    assert_eq!(
        observed.state(),
        LedgerState::Unknown {
            confirmed_amount: u64::MAX,
            unknown_remaining: UpperBound::Known(1)
        }
    );
    assert!(observed.bound_breached());
    assert_eq!(
        observed.contribution(),
        Err(ResourceError::ArithmeticOverflow)
    );
    assert_eq!(
        upper_service_cost(u64::MAX, 2),
        Err(ResourceError::ArithmeticOverflow)
    );
    assert_eq!(upper_service_cost(2, 300_000), Ok(600_000));
}

#[test]
fn windows_dimensions_and_known_facts_are_not_silently_changed() {
    let cap = limit(40, ExplicitLimit::Finite(100));
    let request = request(&cap, 100, UpperBound::Known(80));
    for time in [-1, 1000] {
        assert_eq!(
            evaluate_reservation(&cap, &[], &request, time),
            Err(ResourceError::WindowMismatch)
        );
    }
    let mut wrong = request.clone();
    wrong.kind = ResourceKind::InputUnits;
    assert_eq!(
        evaluate_reservation(&cap, &[], &wrong, 500),
        Err(ResourceError::LimitMismatch)
    );
    let settled = observe(
        &reserve(&cap, 100, 80),
        200,
        ObservationKind::Settled {
            confirmed_amount: 30,
        },
    )
    .expect("settled")
    .position;
    assert_eq!(
        observe(
            &settled,
            201,
            ObservationKind::Settled {
                confirmed_amount: 29
            }
        ),
        Err(ResourceError::InvalidObservationTransition)
    );
    assert_eq!(
        observe(&settled, 202, ObservationKind::OutcomeUnknown),
        Err(ResourceError::InvalidObservationTransition)
    );
}

#[test]
fn token_observation_cannot_settle_money_under_the_same_reservation() {
    let mut money_limit = limit(40, ExplicitLimit::Finite(500));
    money_limit.kind = ResourceKind::MoneyMinorUnits { currency: id(900) };
    let mut token_limit = limit(41, ExplicitLimit::Finite(100_000));
    token_limit.kind = ResourceKind::OutputUnits;
    let money = reserve(&money_limit, 100, 500);
    let tokens = reserve(&token_limit, 100, 100_000);
    assert_eq!(
        money.request().reservation_id,
        tokens.request().reservation_id
    );
    let token_observation = ResourceObservation {
        observation_id: id(200),
        reservation_id: tokens.request().reservation_id,
        ledger_key: tokens.request().key,
        resource_kind: tokens.request().kind,
        kind: ObservationKind::Settled {
            confirmed_amount: 73_000,
        },
    };
    assert_eq!(
        apply_observation(&money, token_observation),
        Err(ResourceError::ObservationMismatch)
    );
}

#[test]
fn data_class_wire_is_the_canonical_sensitivity_taxonomy() {
    for name in ["Public", "Local", "Personal", "Restricted", "Secret"] {
        let wire = format!("\"{name}\"");
        let parsed = serde_json::from_str::<DataClass>(&wire)
            .unwrap_or_else(|error| panic!("canonical sensitivity {name} was rejected: {error}"));
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }
    for old_or_representation in [
        "OwnerPrivate",
        "Confidential",
        "SecretReference",
        "OpaqueSecretHandle",
        "Unknown",
    ] {
        assert!(
            serde_json::from_str::<DataClass>(&format!("\"{old_or_representation}\"")).is_err()
        );
    }
}

#[test]
fn every_observation_requires_exact_binding_even_on_the_replay_path() {
    let mut slots_limit = limit(40, ExplicitLimit::Finite(100));
    slots_limit.kind = ResourceKind::ConcurrentSlots;
    let position = reserve(&slots_limit, 100, 50);
    for kind in [
        ObservationKind::OutcomeUnknown,
        ObservationKind::Partial {
            confirmed_amount: 10,
            unknown_remaining: UpperBound::Known(40),
        },
        ObservationKind::Settled {
            confirmed_amount: 10,
        },
        ObservationKind::NotDispatchedConfirmed,
        ObservationKind::ProcessTerminationConfirmed,
    ] {
        let valid = ResourceObservation {
            observation_id: id(200),
            reservation_id: position.request().reservation_id,
            ledger_key: position.request().key,
            resource_kind: position.request().kind,
            kind,
        };
        let applied = apply_observation(&position, valid).expect("correct position and dimension");
        let mutations = [
            ResourceObservation {
                reservation_id: id(999),
                ..valid
            },
            ResourceObservation {
                ledger_key: LedgerKey {
                    logical_operation_id: id(999),
                    ..valid.ledger_key
                },
                ..valid
            },
            ResourceObservation {
                ledger_key: LedgerKey {
                    limit_id: id(999),
                    ..valid.ledger_key
                },
                ..valid
            },
            ResourceObservation {
                ledger_key: LedgerKey {
                    window_id: id(999),
                    ..valid.ledger_key
                },
                ..valid
            },
            ResourceObservation {
                resource_kind: ResourceKind::InputUnits,
                ..valid
            },
            ResourceObservation {
                resource_kind: ResourceKind::OutputUnits,
                ..valid
            },
            ResourceObservation {
                resource_kind: ResourceKind::MoneyMinorUnits { currency: id(900) },
                ..valid
            },
        ];
        for wrong in mutations {
            assert_eq!(
                apply_observation(&position, wrong),
                Err(ResourceError::ObservationMismatch)
            );
            // The matching ID cannot bypass a changed binding on replay.
            assert_eq!(
                apply_observation(&applied.position, wrong),
                Err(ResourceError::ObservationMismatch)
            );
        }
        let replay = apply_observation(&applied.position, valid).expect("exact repeat");
        assert!(replay.already_applied);
        assert_eq!(replay.position, applied.position);
    }
    assert_eq!(position.state(), LedgerState::Reserved);
    assert_eq!(position.row_version(), version(1));
    assert_eq!(position.contribution(), Ok(UpperBound::Known(50)));
}

#[test]
fn same_reservation_does_not_authorize_currency_limit_or_window_fanout() {
    let mut eur_limit = limit(40, ExplicitLimit::Finite(500));
    eur_limit.kind = ResourceKind::MoneyMinorUnits { currency: id(900) };
    let eur = reserve(&eur_limit, 100, 500);
    let eur_observation = ResourceObservation {
        observation_id: id(200),
        reservation_id: eur.request().reservation_id,
        ledger_key: eur.request().key,
        resource_kind: eur.request().kind,
        kind: ObservationKind::Settled {
            confirmed_amount: 25,
        },
    };
    let wrong_currency = ResourceObservation {
        resource_kind: ResourceKind::MoneyMinorUnits { currency: id(901) },
        ..eur_observation
    };
    assert_eq!(
        apply_observation(&eur, wrong_currency),
        Err(ResourceError::ObservationMismatch)
    );
    let other_limits = [
        ResourceLimit {
            limit_id: id(41),
            ..eur_limit.clone()
        },
        ResourceLimit {
            window_id: id(51),
            ..eur_limit.clone()
        },
        ResourceLimit {
            limit_id: id(42),
            kind: ResourceKind::MoneyMinorUnits { currency: id(901) },
            ..eur_limit
        },
    ];
    for other_limit in other_limits {
        let other = reserve(&other_limit, 100, 500);
        assert_eq!(other.request().reservation_id, eur.request().reservation_id);
        assert_eq!(
            apply_observation(&other, eur_observation),
            Err(ResourceError::ObservationMismatch)
        );
        let exact_other_observation = ResourceObservation {
            observation_id: id(201),
            ledger_key: other.request().key,
            resource_kind: other.request().kind,
            ..eur_observation
        };
        let exact = apply_observation(&other, exact_other_observation)
            .expect("explicit application to another applicable position");
        assert_eq!(exact.position.contribution(), Ok(UpperBound::Known(25)));
    }
}

#[test]
fn sensitivity_sets_are_explicit_subsets_without_implicit_rank_or_reference_alias() {
    let classes = [
        DataClass::Public,
        DataClass::Local,
        DataClass::Personal,
        DataClass::Restricted,
        DataClass::Secret,
    ];
    let mut base = lease(10, 20).scope;
    base.action_kind = ActionKind::ReadWorkspaceFile;
    base.resource = CanonicalResource::WorkspaceFile {
        workspace_id: id(70),
        binding_revision: version(1),
        object_id: id(71),
    };
    base.recipient = None;
    for parent_mask in 0_u32..32 {
        let mut parent = base.clone();
        parent.data_classes = classes
            .iter()
            .enumerate()
            .filter_map(|(bit, class)| ((parent_mask & (1 << bit)) != 0).then_some(*class))
            .collect();
        for child_mask in 0_u32..32 {
            let mut child = base.clone();
            child.data_classes = classes
                .iter()
                .enumerate()
                .filter_map(|(bit, class)| ((child_mask & (1 << bit)) != 0).then_some(*class))
                .collect();
            assert_eq!(
                is_scope_subset(&parent, &child).is_ok(),
                child_mask & !parent_mask == 0
            );
        }
    }
}

#[test]
fn secret_content_never_becomes_a_permitted_model_payload() {
    let mut permitted = lease(10, 20);
    permitted.scope.data_classes = BTreeSet::from([
        DataClass::Public,
        DataClass::Local,
        DataClass::Personal,
        DataClass::Restricted,
    ]);
    assert_eq!(
        verify_lease(&permitted, version(1), &snapshot(&[&permitted]), now()),
        Ok(())
    );
    let mut forbidden = permitted.clone();
    forbidden.scope.data_classes.insert(DataClass::Secret);
    assert_eq!(
        verify_lease(&forbidden, version(1), &snapshot(&[&forbidden]), now()),
        Err(PermissionError::SecretDisclosureDenied)
    );
    assert_eq!(
        is_scope_subset(&forbidden.scope, &permitted.scope),
        Err(PermissionError::SecretDisclosureDenied)
    );
    assert_eq!(
        is_scope_subset(&permitted.scope, &forbidden.scope),
        Err(PermissionError::SecretDisclosureDenied)
    );
    // No model-side secret handle or alternate representation is introduced.
    assert!(serde_json::from_str::<DataClass>("\"SecretReference\"").is_err());
}
