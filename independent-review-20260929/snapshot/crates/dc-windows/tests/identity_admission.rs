//! Synthetic C-12 tests only. No Windows API, handle, pipe, token or
//! AppContainer profile is exercised. These are deterministic typed checks and
//! must not be reported as process-isolation or OS-identity evidence.

use dc_domain::ids::{EntityId, RunGeneration};
use dc_windows::admission::{
    AdmissionError as GateError, AdmissionGate, GateState, enter_admission, leave_admission,
};
use dc_windows::identity::{PeerAuthenticationError, PeerIdentity, PeerRole, authenticate_peer};

fn id(value: u128) -> EntityId {
    EntityId::from_u128(0x019f_0000_0000_7000_8000_0000_0000_0000 | value)
        .expect("synthetic UUIDv7 fixture")
}

fn gv(value: u64) -> RunGeneration {
    RunGeneration::new(value).expect("nonzero fixture")
}

fn peer(role: PeerRole, install: u128, generation: u64) -> PeerIdentity {
    PeerIdentity {
        role,
        installation_id: id(install),
        run_generation: gv(generation),
    }
}

#[test]
fn stale_identity_is_denied_deterministically() {
    let expected = peer(PeerRole::Broker, 1, 2);
    // A same-role, same-installation peer still carrying generation 1 is stale.
    let stale = peer(PeerRole::Broker, 1, 1);
    assert_eq!(
        authenticate_peer(&stale, &expected),
        Err(PeerAuthenticationError::StaleGeneration)
    );
    // A merely different (newer-looking) generation also fails closed.
    let future = peer(PeerRole::Broker, 1, 3);
    assert_eq!(
        authenticate_peer(&future, &expected),
        Err(PeerAuthenticationError::StaleGeneration)
    );
}

#[test]
fn matching_identity_accepted_and_mismatch_kinds_denied() {
    let expected = peer(PeerRole::Watchdog, 1, 1);
    assert_eq!(authenticate_peer(&expected, &expected), Ok(()));
    assert_eq!(
        authenticate_peer(&peer(PeerRole::Worker, 1, 1), &expected),
        Err(PeerAuthenticationError::PeerMismatch)
    );
    assert_eq!(
        authenticate_peer(&peer(PeerRole::Watchdog, 2, 1), &expected),
        Err(PeerAuthenticationError::WrongInstallation)
    );
}

#[test]
fn gate_admits_one_holder_and_refuses_stale_generation() {
    let mut gate = AdmissionGate::idle();
    let holder = peer(PeerRole::Core, 1, 2);
    let token = enter_admission(&mut gate, holder, gv(2)).expect("claim");
    assert_eq!(gate.state(), GateState::Held);
    assert!(matches!(
        enter_admission(&mut gate, holder, gv(2)),
        Err(GateError::GateBusy)
    ));
    // Token is consumed by leave_admission (non-Copy semantics)
    assert_eq!(leave_admission(&mut gate, token), Ok(()));
    assert_eq!(gate.state(), GateState::Idle);
    // Second release attempt with same token is impossible (token moved)
    // This is the P1-ADMISSION-TOKEN-COPY fix: compile-time prevention of reuse
    // Gate is idle again: a holder whose generation is already older is refused.
    assert_eq!(
        enter_admission(&mut gate, holder, gv(1)),
        Err(GateError::StaleGeneration)
    );
}
