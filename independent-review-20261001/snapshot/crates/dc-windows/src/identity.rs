//! C-12 typed peer-identity comparison. No OS acquisition lives here.
//!
//! This module compares a typed identity claim against an expected authority
//! view. It does NOT open a named pipe, read a process token or resolve an
//! AppContainer SID, so a successful comparison proves nothing about the real
//! Windows principal on the other end of a channel. The OS-side acquisition
//! adapter must feed this comparison with a principal it actually verified
//! (contract 17 §7); only then is the pairing meaningful.

use dc_domain::ids::{EntityId, RunGeneration};
use thiserror::Error;

/// Closed peer roles. A claimed role is an enum variant, never model text, and
/// the "role -> permitted command kinds" mapping belongs to C-02's closed table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PeerRole {
    Core,
    Broker,
    Watchdog,
    Worker,
    Desktop,
}

/// A peer identity supplied by the verified channel layer. The fields are the
/// typed, comparable half of the identity; the OS principal (SID, AppContainer
/// SID, token sequence) is NOT represented here and is NOT proven by this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerIdentity {
    pub role: PeerRole,
    pub installation_id: EntityId,
    pub run_generation: RunGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PeerAuthenticationError {
    #[error("peer role differs from the role expected on this channel")]
    PeerMismatch,
    #[error("peer identity belongs to another installation")]
    WrongInstallation,
    #[error("peer identity is bound to a stale run generation")]
    StaleGeneration,
}

/// The identity half of C-02 `validate_sender`. The caller must already hold a
/// verified channel binding (descriptor or one-time secret handed over at
/// spawn); this function performs the deterministic role/installation/generation
/// checks and refuses any mismatch, including a generation that merely differs.
///
/// Generation equality is required, not merely "not newer": an older *or* newer
/// generation than the current authority view means sender and authority have
/// diverged, which fails closed.
pub fn authenticate_peer(
    claimed: &PeerIdentity,
    expected: &PeerIdentity,
) -> Result<(), PeerAuthenticationError> {
    if claimed.role != expected.role {
        return Err(PeerAuthenticationError::PeerMismatch);
    }
    if claimed.installation_id != expected.installation_id {
        return Err(PeerAuthenticationError::WrongInstallation);
    }
    if claimed.run_generation != expected.run_generation {
        return Err(PeerAuthenticationError::StaleGeneration);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn matching_identity_is_accepted() {
        let expected = peer(PeerRole::Broker, 1, 2);
        assert_eq!(authenticate_peer(&expected, &expected), Ok(()));
    }

    #[test]
    fn stale_generation_is_denied_deterministically() {
        let expected = peer(PeerRole::Broker, 1, 2);
        let stale = peer(PeerRole::Broker, 1, 1);
        assert_eq!(
            authenticate_peer(&stale, &expected),
            Err(PeerAuthenticationError::StaleGeneration)
        );
        let future = peer(PeerRole::Broker, 1, 3);
        assert_eq!(
            authenticate_peer(&future, &expected),
            Err(PeerAuthenticationError::StaleGeneration)
        );
    }

    #[test]
    fn role_and_installation_mismatch_are_distinct_denials() {
        let expected = peer(PeerRole::Watchdog, 1, 1);
        assert_eq!(
            authenticate_peer(&peer(PeerRole::Worker, 1, 1), &expected),
            Err(PeerAuthenticationError::PeerMismatch)
        );
        assert_eq!(
            authenticate_peer(&peer(PeerRole::Watchdog, 2, 1), &expected),
            Err(PeerAuthenticationError::WrongInstallation)
        );
    }
}
