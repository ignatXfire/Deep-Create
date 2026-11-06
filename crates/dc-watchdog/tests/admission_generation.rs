//! Synthetic C-10 tests only. No authority.anchor file, SQLite transaction,
//! native gate handle, dispatched effect (D) or committed row (C) is exercised.
//! These deterministic checks prove the typed denial logic, not OS authority.

use dc_domain::ids::{EntityId, RowVersion, RunGeneration};
use dc_domain::permission::{AuthoritySnapshot, CurrentFence, FenceScope};
use dc_watchdog::admission::{
    AdmissionError, begin_fenced_operation, establish_fence, finish_fenced_operation,
    verify_at_boundary,
};
use dc_watchdog::authority_anchor::{AuthorityAnchor, AuthorityAnchorError, next_run_generation};
use dc_windows::admission::AdmissionGate;
use dc_windows::identity::{PeerIdentity, PeerRole};

fn id(value: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | value)
        .expect("synthetic UUIDv7 fixture")
}

fn gv(value: u64) -> RunGeneration {
    RunGeneration::new(value).expect("nonzero fixture")
}

fn ver(value: u64) -> RowVersion {
    RowVersion::new(value).expect("nonzero fixture")
}

fn peer(role: PeerRole, install: u128, generation: u64) -> PeerIdentity {
    PeerIdentity {
        role,
        installation_id: id(install),
        run_generation: gv(generation),
    }
}

fn snapshot(
    install: u128,
    epoch: u64,
    generation: u64,
    fences: &[(FenceScope, u64)],
) -> AuthoritySnapshot {
    AuthoritySnapshot {
        installation_id: id(install),
        authority_epoch: gv(epoch),
        run_generation: gv(generation),
        fences: fences
            .iter()
            .map(|(scope, version)| {
                (
                    *scope,
                    CurrentFence {
                        version: ver(*version),
                        closed: false,
                    },
                )
            })
            .collect(),
    }
}

#[test]
fn stale_generation_identity_is_denied_at_the_boundary() {
    // One accepted start advances generation 1 -> 2 (C-10 next_run_generation).
    let anchor = AuthorityAnchor {
        installation_id: id(1),
        authority_epoch: gv(1),
        run_generation: gv(1),
    };
    assert_eq!(next_run_generation(&anchor, false).expect("advance"), gv(2));

    let authority = snapshot(
        1,
        1,
        2,
        &[
            (FenceScope::RootWork(id(10)), 1),
            (FenceScope::Lease(id(20)), 1),
        ],
    );
    let expected_peer = peer(PeerRole::Broker, 1, 2);
    let support = [FenceScope::RootWork(id(10)), FenceScope::Lease(id(20))];
    let mut gate = AdmissionGate::idle();

    // A same-role, same-installation peer carrying the stale generation 1.
    let stale = peer(PeerRole::Broker, 1, 1);
    assert_eq!(
        begin_fenced_operation(&mut gate, stale, &expected_peer, &authority, &support).map(|_| ()),
        Err(AdmissionError::StaleGeneration)
    );

    // The genuinely current identity is admitted and the gate stays usable.
    let held = begin_fenced_operation(
        &mut gate,
        expected_peer,
        &expected_peer,
        &authority,
        &support,
    )
    .expect("admit");
    assert!(finish_fenced_operation(&mut gate, held).is_ok());
}

#[test]
fn fence_established_after_admission_is_denied_at_recheck() {
    let lease_scope = FenceScope::Lease(id(20));
    let authority = snapshot(
        1,
        1,
        1,
        &[(FenceScope::RootWork(id(10)), 1), (lease_scope, 1)],
    );
    let support = [FenceScope::RootWork(id(10)), lease_scope];
    let expected_peer = peer(PeerRole::Core, 1, 1);
    let mut gate = AdmissionGate::idle();
    let held = begin_fenced_operation(
        &mut gate,
        expected_peer,
        &expected_peer,
        &authority,
        &support,
    )
    .expect("admit");

    // Re-check passes against the unchanged view.
    assert_eq!(verify_at_boundary(&held.receipt, &authority), Ok(()));

    // A trusted revoke closes one scope and bumps its fence version (F).
    let revoked = establish_fence(&authority, lease_scope).expect("revoke");
    assert_eq!(
        verify_at_boundary(&held.receipt, &revoked),
        Err(AdmissionError::FenceChanged)
    );

    // The admission is still releaseable by its actual holder.
    assert!(finish_fenced_operation(&mut gate, held).is_ok());
}

#[test]
fn unrelated_work_revoke_does_not_invalidate_the_held_admission() {
    // Audit regression: the admission snapshot must hold the lease support
    // set, not the whole fence table. Work B's fences are added after A's
    // admission, then B is revoked; A's re-check must still pass and A's own
    // fences stay at their captured versions (contract 18 §7.5).
    let work_a = FenceScope::RootWork(id(10));
    let unit_a = FenceScope::Unit(id(11));
    let lease_a = FenceScope::Lease(id(20));
    let work_b = FenceScope::RootWork(id(30));
    let lease_b = FenceScope::Lease(id(40));
    let support_a = [work_a, unit_a, lease_a];

    let authority = snapshot(1, 1, 1, &[(work_a, 1), (unit_a, 1), (lease_a, 1)]);
    let expected_peer = peer(PeerRole::Core, 1, 1);
    let mut gate = AdmissionGate::idle();
    let held = begin_fenced_operation(
        &mut gate,
        expected_peer,
        &expected_peer,
        &authority,
        &support_a,
    )
    .expect("admit A");

    // Work B's fences appear in the authority view after A's admission.
    let mut with_b = authority.clone();
    with_b.fences.insert(
        work_b,
        CurrentFence {
            version: ver(1),
            closed: false,
        },
    );
    with_b.fences.insert(
        lease_b,
        CurrentFence {
            version: ver(1),
            closed: false,
        },
    );
    assert_eq!(verify_at_boundary(&held.receipt, &with_b), Ok(()));

    // B is then revoked: its fences close and advance. A must be untouched.
    let revoked_b = establish_fence(&with_b, work_b).expect("revoke B work");
    let revoked_b = establish_fence(&revoked_b, lease_b).expect("revoke B lease");
    assert_eq!(verify_at_boundary(&held.receipt, &revoked_b), Ok(()));

    // And A's own revoke still denies A deterministically.
    let revoked_a = establish_fence(&revoked_b, lease_a).expect("revoke A lease");
    assert_eq!(
        verify_at_boundary(&held.receipt, &revoked_a),
        Err(AdmissionError::FenceChanged)
    );

    assert!(finish_fenced_operation(&mut gate, held).is_ok());
}

#[test]
fn support_fence_absent_at_entry_is_refused_before_any_gate_use() {
    let authority = snapshot(1, 1, 1, &[(FenceScope::RootWork(id(10)), 1)]);
    let expected_peer = peer(PeerRole::Core, 1, 1);
    let mut gate = AdmissionGate::idle();
    // The lease's support set names a fence the authority does not know.
    let support = [FenceScope::RootWork(id(10)), FenceScope::Lease(id(99))];
    assert_eq!(
        begin_fenced_operation(
            &mut gate,
            expected_peer,
            &expected_peer,
            &authority,
            &support
        )
        .map(|_| ()),
        Err(AdmissionError::FenceChanged)
    );
    // The gate was not left claimed by the refused entry.
    let held = begin_fenced_operation(
        &mut gate,
        expected_peer,
        &expected_peer,
        &authority,
        &[FenceScope::RootWork(id(10))],
    )
    .expect("gate still usable");
    assert!(finish_fenced_operation(&mut gate, held).is_ok());
}

#[test]
fn epoch_or_generation_advance_denies_the_held_admission() {
    let authority = snapshot(1, 1, 1, &[]);
    let expected_peer = peer(PeerRole::Broker, 1, 1);
    let mut gate = AdmissionGate::idle();
    let held = begin_fenced_operation(&mut gate, expected_peer, &expected_peer, &authority, &[])
        .expect("admit");

    assert_eq!(
        verify_at_boundary(&held.receipt, &snapshot(1, 1, 2, &[])),
        Err(AdmissionError::StaleGeneration)
    );
    assert_eq!(
        verify_at_boundary(&held.receipt, &snapshot(1, 2, 1, &[])),
        Err(AdmissionError::EpochMismatch)
    );

    assert!(finish_fenced_operation(&mut gate, held).is_ok());
}

#[test]
fn role_and_installation_mismatch_are_denied_distinctly() {
    let authority = snapshot(1, 1, 1, &[]);
    let expected_peer = peer(PeerRole::Worker, 1, 1);
    let mut gate = AdmissionGate::idle();

    assert_eq!(
        begin_fenced_operation(
            &mut gate,
            peer(PeerRole::Core, 1, 1),
            &expected_peer,
            &authority,
            &[]
        )
        .map(|_| ()),
        Err(AdmissionError::PeerMismatch)
    );
    assert_eq!(
        begin_fenced_operation(
            &mut gate,
            peer(PeerRole::Worker, 2, 1),
            &expected_peer,
            &authority,
            &[]
        )
        .map(|_| ()),
        Err(AdmissionError::WrongInstallation)
    );
}

#[test]
fn generation_advances_only_on_a_single_accepted_start() {
    let anchor = AuthorityAnchor {
        installation_id: id(1),
        authority_epoch: gv(1),
        run_generation: gv(1),
    };
    assert_eq!(
        next_run_generation(&anchor, true),
        Err(AuthorityAnchorError::AlreadyRunning)
    );
    let at_max = AuthorityAnchor {
        run_generation: RunGeneration::new(u64::MAX).expect("nonzero"),
        ..anchor.clone()
    };
    assert_eq!(
        next_run_generation(&at_max, false),
        Err(AuthorityAnchorError::ArithmeticOverflow)
    );
}
