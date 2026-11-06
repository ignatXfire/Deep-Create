//! Identities are supplied by an authenticated boundary, never generated here.

use std::fmt;
use std::num::{NonZeroU32, NonZeroU64, NonZeroU128};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("identity must be UUIDv7 encoded as exactly 32 lowercase hexadecimal digits")]
pub struct InvalidEntityId;

/// UUIDv7 bit layout (RFC 9562 section 5.7). Wire form is text: JavaScript must
/// not round a 128-bit identity. Validation does NOT prove origin, uniqueness,
/// clock accuracy or authority; generation belongs to the trusted owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EntityId(NonZeroU128);

impl EntityId {
    pub const fn from_u128(value: u128) -> Option<Self> {
        if ((value >> 76) & 0xf) != 7 || ((value >> 62) & 0x3) != 2 {
            return None;
        }
        match NonZeroU128::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl TryFrom<String> for EntityId {
    type Error = InvalidEntityId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 32
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(InvalidEntityId);
        }
        let parsed = u128::from_str_radix(&value, 16).map_err(|_| InvalidEntityId)?;
        Self::from_u128(parsed).ok_or(InvalidEntityId)
    }
}

impl From<EntityId> for String {
    fn from(value: EntityId) -> Self {
        value.to_string()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

pub type UnitRevision = NonZeroU32;
pub type RowVersion = NonZeroU64;
pub type AuthorityEpoch = NonZeroU64;
pub type RunGeneration = NonZeroU64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_round_trips_without_javascript_number_loss() {
        let id = EntityId::from_u128(0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f)
            .expect("RFC 9562 UUIDv7 fixture");
        let text = serde_json::to_string(&id).expect("serialize");
        assert_eq!(text, "\"017f22e279b07cc398c4dc0c0c07398f\"");
        assert_eq!(serde_json::from_str::<EntityId>(&text).expect("parse"), id);
    }

    #[test]
    fn rejects_zero_noncanonical_and_numeric_wire_ids() {
        for input in [
            "\"00000000000000000000000000000000\"",
            "\"1\"",
            "1",
            "\"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\"",
        ] {
            assert!(serde_json::from_str::<EntityId>(input).is_err());
        }
    }
}
