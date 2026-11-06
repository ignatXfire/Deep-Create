//! C-10 `authority.anchor` value and generation arithmetic (typed, not persisted).
//!
//! The watchdog owns the greatest confirmed authority epoch and the current run
//! generation. This module models them as plain values plus the monotonic
//! transitions. Durable storage in `authority.anchor` (WAL + FULL SQLite) and
//! the anti-rollback anchor are C-03/C-10 integration work and are NOT proven
//! here: a value constructed from `NonZeroU64` carries no persistence or
//! anti-rollback guarantee.

use dc_domain::ids::{AuthorityEpoch, EntityId, RunGeneration};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityAnchor {
    pub installation_id: EntityId,
    pub authority_epoch: AuthorityEpoch,
    pub run_generation: RunGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AuthorityAnchorError {
    #[error("a run is already in progress; no new generation is accepted")]
    AlreadyRunning,
    #[error("integer authority counter overflow")]
    ArithmeticOverflow,
}

/// C-10 `next_run_generation`: exactly one accepted core start advances the
/// generation by one. A simultaneous second start is refused. `run_in_progress`
/// is supplied by the watchdog's own live state, not inferred from the value.
pub fn next_run_generation(
    anchor: &AuthorityAnchor,
    run_in_progress: bool,
) -> Result<RunGeneration, AuthorityAnchorError> {
    if run_in_progress {
        return Err(AuthorityAnchorError::AlreadyRunning);
    }
    anchor
        .run_generation
        .get()
        .checked_add(1)
        .and_then(RunGeneration::new)
        .ok_or(AuthorityAnchorError::ArithmeticOverflow)
}

/// Global authority-epoch bump for installation-wide guard closure. Individual
/// revoke and work stop use exact fences (see `admission::establish_fence`),
/// never a global epoch.
pub fn advance_authority_epoch(
    anchor: &AuthorityAnchor,
) -> Result<AuthorityAnchor, AuthorityAnchorError> {
    let epoch = anchor
        .authority_epoch
        .get()
        .checked_add(1)
        .and_then(AuthorityEpoch::new)
        .ok_or(AuthorityAnchorError::ArithmeticOverflow)?;
    Ok(AuthorityAnchor {
        authority_epoch: epoch,
        ..anchor.clone()
    })
}
