//! C-01 read binding, NOT accounting, authority or a persistence receipt.
//! C-03 must prove manifest completeness/authenticity and compare these rows
//! atomically with the result. Unknown accounting remains accounted, not freed.

use crate::ids::{EntityId, RowVersion};
use crate::resource::{
    LedgerKey, LedgerPosition, LedgerState, ReservationRequest, ResourceError, ResourceKind,
    UpperBound,
};
use crate::structure::STRUCTURAL_LIMITS_V1;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const MAX_ITEMS: usize = STRUCTURAL_LIMITS_V1.max_collection_items;

/// Exact expected rows from the trusted attempt's complete usage manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedUsagePosition {
    pub key: LedgerKey,
    pub reservation_id: EntityId,
    pub kind: ResourceKind,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsagePositionBinding {
    expected: ExpectedUsagePosition,
    request: ReservationRequest,
    state: LedgerState,
    contribution: UpperBound,
    bound_breached: bool,
    observation_ids: Vec<EntityId>,
}

impl UsagePositionBinding {
    pub fn expected(&self) -> &ExpectedUsagePosition {
        &self.expected
    }
    pub fn request(&self) -> &ReservationRequest {
        &self.request
    }
    pub fn state(&self) -> LedgerState {
        self.state
    }
    pub fn contribution(&self) -> UpperBound {
        self.contribution
    }
    pub fn bound_breached(&self) -> bool {
        self.bound_breached
    }
    pub fn observation_ids(&self) -> &[EntityId] {
        &self.observation_ids
    }
}

/// Immutable evidence references. No constructor or deserialization bypass.
///
/// ```compile_fail
/// let _: dc_domain::usage_binding::UsageBinding = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageBinding {
    attempt_id: EntityId,
    positions: Vec<UsagePositionBinding>,
}

impl UsageBinding {
    pub fn attempt_id(&self) -> EntityId {
        self.attempt_id
    }
    pub fn positions(&self) -> &[UsagePositionBinding] {
        &self.positions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UsageBindingError {
    #[error("usage lists must be nonempty and structurally bounded")]
    EmptyOrOversized,
    #[error("duplicate ledger key in manifest or supplied positions")]
    DuplicateKey,
    #[error("expected and supplied ledger row sets differ")]
    ManifestMismatch,
    #[error("reservation, resource dimension or row version differs")]
    PositionMismatch,
    #[error("ledger row belongs to another attempt")]
    AttemptMismatch,
    #[error("no applied observation for ledger row")]
    NoObservation,
    #[error("contribution arithmetic failed: {0}")]
    Contribution(#[from] ResourceError),
}

/// Bind existing facts without charging or releasing anything. This API handles
/// observed usage only: [] is not evidence that an attempt was free of usage.
pub fn bind_usage(
    manifest: &[ExpectedUsagePosition],
    positions: &[LedgerPosition],
    attempt_id: EntityId,
) -> Result<UsageBinding, UsageBindingError> {
    if manifest.is_empty()
        || positions.is_empty()
        || manifest.len() > MAX_ITEMS
        || positions.len() > MAX_ITEMS
    {
        return Err(UsageBindingError::EmptyOrOversized);
    }
    let mut expected_keys = BTreeSet::new();
    for expected in manifest {
        if !expected_keys.insert(expected.key) {
            return Err(UsageBindingError::DuplicateKey);
        }
    }
    let mut by_key = BTreeMap::new();
    for position in positions {
        if by_key.insert(position.request().key, position).is_some() {
            return Err(UsageBindingError::DuplicateKey);
        }
    }
    if expected_keys.len() != by_key.len() {
        return Err(UsageBindingError::ManifestMismatch);
    }
    let mut bound = Vec::with_capacity(manifest.len());
    for expected in manifest {
        let position = by_key
            .get(&expected.key)
            .ok_or(UsageBindingError::ManifestMismatch)?;
        let request = position.request();
        if request.reservation_id != expected.reservation_id
            || request.kind != expected.kind
            || position.row_version() != expected.row_version
        {
            return Err(UsageBindingError::PositionMismatch);
        }
        if request.attempt_id != attempt_id {
            return Err(UsageBindingError::AttemptMismatch);
        }
        let observation_ids: Vec<_> = position.observation_ids().take(MAX_ITEMS + 1).collect();
        if observation_ids.is_empty() {
            return Err(UsageBindingError::NoObservation);
        }
        if observation_ids.len() > MAX_ITEMS {
            return Err(UsageBindingError::EmptyOrOversized);
        }
        bound.push(UsagePositionBinding {
            expected: expected.clone(),
            request: request.clone(),
            state: position.state(),
            contribution: position.contribution()?,
            bound_breached: position.bound_breached(),
            observation_ids,
        });
    }
    Ok(UsageBinding {
        attempt_id,
        positions: bound,
    })
}
