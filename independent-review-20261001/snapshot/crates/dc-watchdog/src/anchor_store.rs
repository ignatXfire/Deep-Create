//! C-10 `authority.anchor` durable persistence: the anti-rollback record.
//!
//! This module wires the watchdog's authority epoch / run generation into a
//! durable store. On startup the watchdog loads the last anchor; on every
//! epoch advance or new run generation it writes the new anchor before any
//! dispatch is admitted.
//!
//! The `Store` generic is the `dc_store::Stage3Backend` interface, which is
//! implemented by `SQLiteStore` with WAL + synchronous=FULL. The in-memory
//! `MemoryStore` does not implement it — that is intentional, because a
//! crash-recovery anchor that lives only in memory is not a crash-recovery
//! anchor.
//!
//! What is proven here: the read/write round-trip and the monotonic
//! anti-rollback invariant (a loaded anchor never goes backwards).
//! What remains integration work: the live watchdog loop driving these calls
//! at the right moments, and the multi-store consistency story with dc-core.

use dc_domain::ids::{AuthorityEpoch, EntityId, RunGeneration};
use dc_store::{Stage3Backend, StoreError};
use thiserror::Error;

use crate::authority_anchor::{AuthorityAnchor, AuthorityAnchorError};

/// Errors specific to durable anchor operations.
#[derive(Debug, Error)]
pub enum AnchorStoreError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),

    #[error("anchor state is malformed: {0}")]
    Malformed(String),

    #[error("anti-rollback violation: stored epoch {stored} > new epoch {new}")]
    AntiRollback { stored: u64, new: u64 },

    #[error("authority arithmetic: {0}")]
    Arithmetic(#[from] AuthorityAnchorError),
}

/// The fence scope under which the watchdog's own anchor is stored.
/// This is intentionally not a user-controllable scope — it is the watchdog's
/// own anti-rollback record.
pub const WATCHDOG_ANCHOR_SCOPE: &str = "watchdog.authority.anchor";

/// Serialize an anchor to a compact byte form for the Stage-3 state blob.
/// Format: "epoch=<u64>;generation=<u64>" — human-inspectable, versionable.
fn serialize_anchor(anchor: &AuthorityAnchor) -> Vec<u8> {
    format!(
        "epoch={};generation={}",
        anchor.authority_epoch.get(),
        anchor.run_generation.get()
    )
    .into_bytes()
}

/// Deserialize an anchor from the state blob. Returns `Malformed` on any
/// deviation from the expected format — we never guess.
fn deserialize_anchor(
    data: &[u8],
    _installation_id: EntityId,
) -> Result<(AuthorityEpoch, RunGeneration), AnchorStoreError> {
    let s = std::str::from_utf8(data)
        .map_err(|e| AnchorStoreError::Malformed(format!("not utf8: {e}")))?;

    let mut epoch = None;
    let mut generation = None;
    for part in s.split(';') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("epoch"), Some(v)) => {
                epoch = Some(
                    v.parse::<u64>()
                        .ok()
                        .and_then(AuthorityEpoch::new)
                        .ok_or_else(|| AnchorStoreError::Malformed(format!("bad epoch: {v}")))?,
                );
            }
            (Some("generation"), Some(v)) => {
                generation = Some(
                    v.parse::<u64>()
                        .ok()
                        .and_then(RunGeneration::new)
                        .ok_or_else(|| {
                            AnchorStoreError::Malformed(format!("bad generation: {v}"))
                        })?,
                );
            }
            _ => {}
        }
    }

    match (epoch, generation) {
        (Some(e), Some(g)) => Ok((e, g)),
        _ => Err(AnchorStoreError::Malformed(format!(
            "missing fields in: {s}"
        ))),
    }
}

/// Load the persisted authority anchor, if one exists.
///
/// Returns `None` on a fresh installation (no prior anchor). The caller is
/// responsible for constructing the initial epoch-1 anchor in that case.
pub fn load_anchor<S: Stage3Backend>(
    store: &S,
    installation_id: &EntityId,
) -> Result<Option<AuthorityAnchor>, AnchorStoreError> {
    let record = store.latest_authority_anchor(installation_id, WATCHDOG_ANCHOR_SCOPE)?;
    match record {
        None => Ok(None),
        Some(rec) => {
            let (epoch, generation) = deserialize_anchor(&rec.state, *installation_id)?;
            Ok(Some(AuthorityAnchor {
                installation_id: *installation_id,
                authority_epoch: epoch,
                run_generation: generation,
            }))
        }
    }
}

/// Persist a new authority anchor, enforcing the anti-rollback invariant.
///
/// The stored epoch must never exceed the new epoch — going backwards would
/// mean a rollback attack or a corrupted store, and we refuse to proceed.
/// The caller supplies a `RowVersion` for the CAS write; the anchor's own
/// logical version is the epoch itself.
pub fn persist_anchor<S: Stage3Backend>(
    store: &mut S,
    anchor: &AuthorityAnchor,
) -> Result<(), AnchorStoreError> {
    // Anti-rollback: refuse to write an epoch lower than what is stored.
    if let Some(existing) =
        store.latest_authority_anchor(&anchor.installation_id, WATCHDOG_ANCHOR_SCOPE)?
    {
        let (stored_epoch, _) = deserialize_anchor(&existing.state, anchor.installation_id)?;
        if stored_epoch.get() > anchor.authority_epoch.get() {
            return Err(AnchorStoreError::AntiRollback {
                stored: stored_epoch.get(),
                new: anchor.authority_epoch.get(),
            });
        }
    }

    let state = serialize_anchor(anchor);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    store.put_authority_anchor(&dc_store::AuthorityAnchor {
        anchor_id: format!("watchdog-{}", anchor.installation_id),
        entity_id: anchor.installation_id,
        fence_scope: WATCHDOG_ANCHOR_SCOPE.to_string(),
        row_version: anchor.authority_epoch.get(),
        state,
        created_at: now,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::ids::{AuthorityEpoch, EntityId, RunGeneration};
    use dc_store::{ExpectedVersion, SQLiteStore, StoreBackend};

    fn test_installation_id(seed: u64) -> EntityId {
        let mut value = (seed as u128) & 0x3FFFFFFFFFFFFFFF;
        value |= 0x7 << 76;
        value |= 0x2 << 62;
        EntityId::from_u128(value).unwrap()
    }

    fn make_anchor(installation_id: EntityId, epoch: u64, generation: u64) -> AuthorityAnchor {
        AuthorityAnchor {
            installation_id,
            authority_epoch: AuthorityEpoch::new(epoch).unwrap(),
            run_generation: RunGeneration::new(generation).unwrap(),
        }
    }

    #[test]
    fn anchor_persist_load_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "dc-watchdog-anchor-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("anchor.db");

        let iid = test_installation_id(1);

        // Phase 1: persist
        {
            let mut store = SQLiteStore::new(db_path.to_str().unwrap());
            store.initialize(std::path::Path::new(&db_path)).unwrap();
            // Stage-3 tables have FK to entities(id) — insert a parent row.
            store
                .write_bytes(&iid, b"installation", ExpectedVersion::MustNotExist)
                .unwrap();

            let anchor = make_anchor(iid, 5, 12);
            persist_anchor(&mut store, &anchor).unwrap();
        }

        // Phase 2: reload from disk — proves durability across restart.
        {
            let mut store = SQLiteStore::new(db_path.to_str().unwrap());
            store.initialize(std::path::Path::new(&db_path)).unwrap();

            let loaded = load_anchor(&store, &iid).unwrap().unwrap();
            assert_eq!(loaded.authority_epoch.get(), 5);
            assert_eq!(loaded.run_generation.get(), 12);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn anchor_anti_rollback_refused() {
        let dir = std::env::temp_dir().join(format!(
            "dc-watchdog-anti-rollback-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("anchor.db");

        let iid = test_installation_id(2);

        let mut store = SQLiteStore::new(db_path.to_str().unwrap());
        store.initialize(std::path::Path::new(&db_path)).unwrap();
        store
            .write_bytes(&iid, b"installation", ExpectedVersion::MustNotExist)
            .unwrap();

        // Persist epoch 10
        persist_anchor(&mut store, &make_anchor(iid, 10, 1)).unwrap();

        // Attempt to roll back to epoch 5 — must be refused.
        let err = persist_anchor(&mut store, &make_anchor(iid, 5, 2)).unwrap_err();
        assert!(matches!(
            err,
            AnchorStoreError::AntiRollback { stored: 10, new: 5 }
        ));

        // Epoch 11 (forward) is accepted.
        persist_anchor(&mut store, &make_anchor(iid, 11, 3)).unwrap();
        let loaded = load_anchor(&store, &iid).unwrap().unwrap();
        assert_eq!(loaded.authority_epoch.get(), 11);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn anchor_malformed_state_rejected() {
        let iid = test_installation_id(3);
        let result = deserialize_anchor(b"garbage", iid);
        assert!(matches!(result, Err(AnchorStoreError::Malformed(_))));

        let result = deserialize_anchor(b"epoch=abc;generation=1", iid);
        assert!(matches!(result, Err(AnchorStoreError::Malformed(_))));
    }
}
