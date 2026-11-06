//! C-03 store: the sole write path for the main database.
//!
//! This crate provides the storage abstraction for Deep Create.
//! No network, clock, or env access — pure storage.
//! SQLite implementation is behind the `sqlite` feature flag.

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "sqlite")]
pub use sqlite::SQLiteStore;

pub mod stage3;
pub use stage3::{
    EvidenceArtifact, JournalEvent, LedgerEntry, ResourceKind, Stage3CanonicalBackend, sha256_hex,
};

use std::path::Path;

use dc_domain::ids::{EntityId, RowVersion};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Storage errors.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage not initialized")]
    NotInitialized,

    #[error("storage already initialized")]
    AlreadyInitialized,

    #[error("entity not found: {0}")]
    NotFound(EntityId),

    #[error("row version conflict: expected {expected}, found {found}")]
    RowVersionConflict {
        expected: RowVersion,
        found: RowVersion,
    },

    #[error("row version conflict: expected no existing row, but found version {found}")]
    RowVersionMustNotExist { found: RowVersion },

    #[error("row version conflict: expected row to exist, but it is absent")]
    RowVersionExpectedMissing,

    #[error("row version exceeds SQLite i64 storage bound: {0}")]
    RowVersionOverflow(u64),

    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    #[error("integrity check failed: {0}")]
    IntegrityFailure(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A stored entity with version tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredEntity<T> {
    /// The entity ID.
    pub id: EntityId,
    /// The current row version.
    pub row_version: RowVersion,
    /// The entity data.
    pub data: T,
    /// When the entity was created (logical timestamp).
    pub created_at: u64,
    /// When the entity was last modified (logical timestamp).
    pub updated_at: u64,
}

/// Precondition on the existing row for a compare-and-swap write.
///
/// Every authoritative write MUST state what it believes the current state is.
/// `Any` is an unconditional overwrite and is reserved for tests and tools that
/// have already serialized access by other means; the single-writer rule in
/// C-03 normally demands `Existing` (update) or `MustNotExist` (create).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedVersion {
    /// No precondition. Overwrite whatever is present.
    Any,
    /// The row must not exist. Fails with `RowVersionMustNotExist` otherwise.
    MustNotExist,
    /// The row must exist with exactly this version. Fails with
    /// `RowVersionConflict` (or `RowVersionExpectedMissing` if absent).
    Existing(RowVersion),
}

/// SQLite stores INTEGER as signed i64. RowVersion is u64; values above
/// `i64::MAX` cannot round-trip through a signed column. Until the storage
/// format moves to an exact unsigned representation (e.g. BLOB), this bound
/// is an enforced storage invariant: writes beyond it fail with
/// `StoreError::RowVersionOverflow` instead of silently wrapping negative.
pub const ROW_VERSION_SQLITE_MAX: u64 = i64::MAX as u64;

/// Map a canonical u64 RowVersion to its SQLite i64 representation,
/// enforcing the storage invariant.
pub(crate) fn row_version_to_sqlite(v: RowVersion) -> Result<i64, StoreError> {
    let raw = v.get();
    if raw > ROW_VERSION_SQLITE_MAX {
        return Err(StoreError::RowVersionOverflow(raw));
    }
    Ok(raw as i64)
}

/// Map a SQLite i64 back to a canonical RowVersion. Negative values indicate
/// a corrupted or foreign-written row and are rejected.
pub(crate) fn row_version_from_sqlite(v: i64) -> Result<RowVersion, StoreError> {
    if v <= 0 {
        return Err(StoreError::IntegrityFailure(format!(
            "invalid stored row version: {}",
            v
        )));
    }
    RowVersion::new(v as u64).ok_or_else(|| StoreError::IntegrityFailure("zero row version".into()))
}

/// The core storage abstraction. All writes go through this trait.
/// Implementations must be ACID-compliant.
///
/// Note: This trait uses generic methods, so it is not dyn-compatible.
/// Use `Store` for a concrete handle that erases the backend type.
pub trait StoreBackend: Send + Sync {
    /// Initialize the storage at the given path.
    fn initialize(&mut self, path: &Path) -> Result<(), StoreError>;

    /// Check if storage is initialized.
    fn is_initialized(&self) -> bool;

    /// Begin a new transaction.
    fn begin_transaction(&mut self) -> Result<(), StoreError>;

    /// Commit the current transaction.
    fn commit(&mut self) -> Result<(), StoreError>;

    /// Rollback the current transaction.
    fn rollback(&mut self) -> Result<(), StoreError>;

    /// Read an entity by ID.
    fn read_bytes(&self, id: &EntityId) -> Result<Option<(RowVersion, Vec<u8>)>, StoreError>;

    /// Write an entity as bytes with a compare-and-swap precondition on the
    /// current row version. Returns the new row version.
    ///
    /// Semantics of `expected`:
    /// - `Any`          — unconditional upsert.
    /// - `MustNotExist` — insert only; `RowVersionMustNotExist` if present.
    /// - `Existing(v)`  — update only if the stored version equals `v`;
    ///   `RowVersionConflict` on mismatch, `RowVersionExpectedMissing` if absent.
    fn write_bytes(
        &mut self,
        id: &EntityId,
        data: &[u8],
        expected: ExpectedVersion,
    ) -> Result<RowVersion, StoreError>;

    /// Delete an entity by ID.
    fn delete(&mut self, id: &EntityId) -> Result<(), StoreError>;

    /// Check if an entity exists.
    fn exists(&self, id: &EntityId) -> Result<bool, StoreError>;

    /// List all entity IDs with a given prefix.
    fn list_by_prefix(&self, prefix: &str) -> Result<Vec<EntityId>, StoreError>;

    /// Run integrity check on the storage.
    fn integrity_check(&self) -> Result<(), StoreError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Stage 3: typed records for events / resources / evidence / recovery / anchor
// ─────────────────────────────────────────────────────────────────────────────

/// An audit-trail event tied to an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub event_type: String,
    pub entity_id: EntityId,
    pub payload: Vec<u8>,
    pub created_at: u64,
}

/// A tracked external resource (file, API token budget, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub resource_id: String,
    pub resource_type: String,
    pub entity_id: EntityId,
    pub state: String,
    pub metadata: Option<Vec<u8>>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// A verification evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub evidence_id: String,
    pub entity_id: EntityId,
    pub evidence_type: String,
    pub verifier_id: String,
    pub payload: Vec<u8>,
    pub verified_at: u64,
    pub expires_at: Option<u64>,
}

/// A crash-recovery checkpoint for an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryCheckpoint {
    pub checkpoint_id: String,
    pub entity_id: EntityId,
    pub checkpoint_type: String,
    pub state: Vec<u8>,
    pub created_at: u64,
}

/// An authority anchor binding an entity to a fence scope at a row version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityAnchor {
    pub anchor_id: String,
    pub entity_id: EntityId,
    pub fence_scope: String,
    pub row_version: u64,
    pub state: Vec<u8>,
    pub created_at: u64,
}

/// Stage-3 extension trait for stores that implement the typed tables.
///
/// This is a separate trait (not part of `StoreBackend`) so the in-memory
/// `MemoryStore` used in domain tests is not forced to implement audit tables
/// it does not need. Only production backends (SQLite) implement this.
pub trait Stage3Backend: StoreBackend {
    fn put_event(&mut self, rec: &EventRecord) -> Result<(), StoreError>;
    fn events_for_entity(&self, entity_id: &EntityId) -> Result<Vec<EventRecord>, StoreError>;

    fn put_resource(&mut self, rec: &ResourceRecord) -> Result<(), StoreError>;
    fn resources_for_entity(&self, entity_id: &EntityId)
    -> Result<Vec<ResourceRecord>, StoreError>;

    fn put_evidence(&mut self, rec: &EvidenceRecord) -> Result<(), StoreError>;
    fn evidence_for_entity(&self, entity_id: &EntityId) -> Result<Vec<EvidenceRecord>, StoreError>;

    fn put_recovery_checkpoint(&mut self, rec: &RecoveryCheckpoint) -> Result<(), StoreError>;
    fn latest_recovery_checkpoint(
        &self,
        entity_id: &EntityId,
    ) -> Result<Option<RecoveryCheckpoint>, StoreError>;

    fn put_authority_anchor(&mut self, rec: &AuthorityAnchor) -> Result<(), StoreError>;
    fn latest_authority_anchor(
        &self,
        entity_id: &EntityId,
        fence_scope: &str,
    ) -> Result<Option<AuthorityAnchor>, StoreError>;
}

/// In-memory storage implementation for testing.
#[derive(Debug, Default)]
pub struct MemoryStore {
    initialized: bool,
    data: std::collections::HashMap<String, (RowVersion, Vec<u8>)>,
    in_transaction: bool,
    transaction_data: Option<std::collections::HashMap<String, (RowVersion, Vec<u8>)>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StoreBackend for MemoryStore {
    fn initialize(&mut self, _path: &Path) -> Result<(), StoreError> {
        if self.initialized {
            return Err(StoreError::AlreadyInitialized);
        }
        self.initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn begin_transaction(&mut self) -> Result<(), StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        if self.in_transaction {
            return Err(StoreError::TransactionFailed(
                "nested transactions not supported".into(),
            ));
        }
        self.in_transaction = true;
        self.transaction_data = Some(self.data.clone());
        Ok(())
    }

    fn commit(&mut self) -> Result<(), StoreError> {
        if !self.in_transaction {
            return Err(StoreError::TransactionFailed(
                "no active transaction".into(),
            ));
        }
        self.in_transaction = false;
        self.transaction_data = None;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), StoreError> {
        if !self.in_transaction {
            return Err(StoreError::TransactionFailed(
                "no active transaction".into(),
            ));
        }
        if let Some(backup) = self.transaction_data.take() {
            self.data = backup;
        }
        self.in_transaction = false;
        Ok(())
    }

    fn read_bytes(&self, id: &EntityId) -> Result<Option<(RowVersion, Vec<u8>)>, StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        let key = id.to_string();
        Ok(self.data.get(&key).map(|(v, b)| (*v, b.clone())))
    }

    fn write_bytes(
        &mut self,
        id: &EntityId,
        data: &[u8],
        expected: ExpectedVersion,
    ) -> Result<RowVersion, StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        let key = id.to_string();
        let current = self.data.get(&key).map(|(v, _)| *v);
        match expected {
            ExpectedVersion::Any => {}
            ExpectedVersion::MustNotExist => {
                if let Some(found) = current {
                    return Err(StoreError::RowVersionMustNotExist { found });
                }
            }
            ExpectedVersion::Existing(expected_v) => match current {
                Some(found) if found == expected_v => {}
                Some(found) => {
                    return Err(StoreError::RowVersionConflict {
                        expected: expected_v,
                        found,
                    });
                }
                None => return Err(StoreError::RowVersionExpectedMissing),
            },
        }
        let new_version = match current {
            Some(version) => version
                .get()
                .checked_add(1)
                .and_then(RowVersion::new)
                .ok_or(StoreError::RowVersionOverflow(u64::MAX))?,
            None => RowVersion::MIN,
        };
        self.data.insert(key, (new_version, data.to_vec()));
        Ok(new_version)
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        self.data.remove(&id.to_string());
        Ok(())
    }

    fn exists(&self, id: &EntityId) -> Result<bool, StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        Ok(self.data.contains_key(&id.to_string()))
    }

    fn list_by_prefix(&self, prefix: &str) -> Result<Vec<EntityId>, StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        let mut result = Vec::new();
        for key in self.data.keys() {
            if key.starts_with(prefix) {
                // Parse UUID from string (32 hex chars)
                if let Ok(id) = EntityId::try_from(key.clone()) {
                    result.push(id);
                }
            }
        }
        Ok(result)
    }

    fn integrity_check(&self) -> Result<(), StoreError> {
        if !self.initialized {
            return Err(StoreError::NotInitialized);
        }
        Ok(())
    }
}

/// The main store handle. This is the only way to access the database.
pub struct Store {
    backend: Box<dyn StoreBackend>,
}

impl Store {
    /// Create a new store with the given backend.
    pub fn new(backend: Box<dyn StoreBackend>) -> Self {
        Self { backend }
    }

    /// Create an in-memory store (for testing).
    pub fn memory() -> Self {
        Self::new(Box::new(MemoryStore::new()))
    }

    /// Initialize the store at the given path.
    pub fn initialize(&mut self, path: &Path) -> Result<(), StoreError> {
        self.backend.initialize(path)
    }

    /// Check if the store is initialized.
    pub fn is_initialized(&self) -> bool {
        self.backend.is_initialized()
    }

    /// Begin a transaction.
    pub fn begin_transaction(&mut self) -> Result<(), StoreError> {
        self.backend.begin_transaction()
    }

    /// Commit the current transaction.
    pub fn commit(&mut self) -> Result<(), StoreError> {
        self.backend.commit()
    }

    /// Rollback the current transaction.
    pub fn rollback(&mut self) -> Result<(), StoreError> {
        self.backend.rollback()
    }

    /// Read an entity.
    pub fn read<T>(&self, id: &EntityId) -> Result<Option<StoredEntity<T>>, StoreError>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.backend.read_bytes(id)? {
            Some((version, bytes)) => {
                let data: T = serde_json::from_slice(&bytes)?;
                Ok(Some(StoredEntity {
                    id: *id,
                    row_version: version,
                    data,
                    created_at: 0,
                    updated_at: 0,
                }))
            }
            None => Ok(None),
        }
    }

    /// Write an entity. Uses `Any` (unconditional upsert) — preserved for
    /// compatibility with existing callers that have no version snapshot.
    /// Prefer `write_with` for authoritative paths.
    pub fn write<T>(&mut self, entity: &StoredEntity<T>) -> Result<RowVersion, StoreError>
    where
        T: Serialize,
    {
        self.write_with(entity, ExpectedVersion::Any)
    }

    /// Write an entity with an explicit compare-and-swap precondition.
    pub fn write_with<T>(
        &mut self,
        entity: &StoredEntity<T>,
        expected: ExpectedVersion,
    ) -> Result<RowVersion, StoreError>
    where
        T: Serialize,
    {
        let bytes = serde_json::to_vec(&entity.data)?;
        self.backend.write_bytes(&entity.id, &bytes, expected)
    }

    /// Write raw bytes with an explicit compare-and-swap precondition.
    /// Exposed for regression tests and for callers that already hold
    /// canonical bytes; prefer `write`/`write_with` for typed entities.
    pub fn write_bytes(
        &mut self,
        id: &EntityId,
        data: &[u8],
        expected: ExpectedVersion,
    ) -> Result<RowVersion, StoreError> {
        self.backend.write_bytes(id, data, expected)
    }

    /// Read raw bytes with the current row version.
    pub fn read_bytes(&self, id: &EntityId) -> Result<Option<(RowVersion, Vec<u8>)>, StoreError> {
        self.backend.read_bytes(id)
    }

    /// Delete an entity.
    pub fn delete(&mut self, id: &EntityId) -> Result<(), StoreError> {
        self.backend.delete(id)
    }

    /// Check if an entity exists.
    pub fn exists(&self, id: &EntityId) -> Result<bool, StoreError> {
        self.backend.exists(id)
    }

    /// Run integrity check.
    pub fn integrity_check(&self) -> Result<(), StoreError> {
        self.backend.integrity_check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::ids::EntityId;
    use std::num::NonZeroU64;

    fn test_id(seed: u128) -> EntityId {
        let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
        EntityId::from_u128(base + seed).expect("valid UUIDv7")
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestData {
        value: String,
    }

    #[test]
    fn memory_store_lifecycle() {
        let mut store = Store::memory();
        assert!(!store.is_initialized());

        store.initialize(Path::new(":memory:")).unwrap();
        assert!(store.is_initialized());
    }

    #[test]
    fn memory_store_write_and_read() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let entity = StoredEntity {
            id,
            row_version: NonZeroU64::MIN,
            data: TestData {
                value: "hello".to_string(),
            },
            created_at: 0,
            updated_at: 0,
        };

        let version = store.write(&entity).unwrap();
        assert_eq!(version, NonZeroU64::MIN);

        let read: Option<StoredEntity<TestData>> = store.read(&id).unwrap();
        assert!(read.is_some());
        assert_eq!(read.unwrap().data.value, "hello");
    }

    #[test]
    fn memory_store_transaction_commit() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        // Begin and commit a transaction
        store.begin_transaction().unwrap();
        store.commit().unwrap();
    }

    #[test]
    fn memory_store_transaction_rollback() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        // Begin and rollback a transaction
        store.begin_transaction().unwrap();
        store.rollback().unwrap();
    }

    #[test]
    fn memory_store_version_increments() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let entity = StoredEntity {
            id,
            row_version: NonZeroU64::MIN,
            data: TestData {
                value: "v1".to_string(),
            },
            created_at: 0,
            updated_at: 0,
        };

        let v1 = store.write(&entity).unwrap();
        let v2 = store.write(&entity).unwrap();
        let v3 = store.write(&entity).unwrap();

        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn memory_store_cas_rejects_stale_writer() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let v1 = store
            .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
            .unwrap();
        assert_eq!(v1, NonZeroU64::MIN);

        // Writer B holds the stale snapshot v1 while writer A advanced the row.
        let _v2 = store
            .write_bytes(&id, b"v2", ExpectedVersion::Existing(v1))
            .unwrap();

        let stale = store.write_bytes(&id, b"stale-overwrite", ExpectedVersion::Existing(v1));
        assert!(matches!(
            stale,
            Err(StoreError::RowVersionConflict { expected, found })
                if expected == v1 && found == _v2
        ));

        // And the losing write must not have landed.
        let (_, bytes) = store.read_bytes(&id).unwrap().unwrap();
        assert_eq!(bytes, b"v2");
    }

    #[test]
    fn memory_store_must_not_exist_rejects_overwrite() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let v1 = store
            .write_bytes(&id, b"first", ExpectedVersion::MustNotExist)
            .unwrap();

        let dup = store.write_bytes(&id, b"second", ExpectedVersion::MustNotExist);
        assert!(matches!(
            dup,
            Err(StoreError::RowVersionMustNotExist { found }) if found == v1
        ));

        let (_, bytes) = store.read_bytes(&id).unwrap().unwrap();
        assert_eq!(bytes, b"first");
    }

    #[test]
    fn memory_store_existing_requires_presence() {
        let mut store = Store::memory();
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let missing = store.write_bytes(&id, b"x", ExpectedVersion::Existing(NonZeroU64::MIN));
        assert!(matches!(
            missing,
            Err(StoreError::RowVersionExpectedMissing)
        ));
    }
}
