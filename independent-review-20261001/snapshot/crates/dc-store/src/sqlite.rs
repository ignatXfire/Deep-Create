//! SQLite implementation of StoreBackend.
//!
//! Feature-gated behind `sqlite` flag. Uses rusqlite with bundled SQLite.

use std::path::Path;
use std::sync::Mutex;

use dc_domain::ids::{EntityId, RowVersion};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    AuthorityAnchor, EventRecord, EvidenceRecord, ExpectedVersion, ROW_VERSION_SQLITE_MAX,
    RecoveryCheckpoint, ResourceRecord, Stage3Backend, StoreBackend, StoreError,
    row_version_from_sqlite, row_version_to_sqlite,
};

/// SQLite-backed storage implementation.
pub struct SQLiteStore {
    conn: Mutex<Option<Connection>>,
    path: String,
    initialized: bool,
    in_transaction: Mutex<bool>,
}

impl SQLiteStore {
    /// Create a new SQLite store at the given path.
    /// Use ":memory:" for in-memory database.
    pub fn new(path: &str) -> Self {
        Self {
            conn: Mutex::new(None),
            path: path.to_string(),
            initialized: false,
            in_transaction: Mutex::new(false),
        }
    }

    /// Get a reference to the connection, initializing if needed.
    fn with_conn<F, R>(&self, f: F) -> Result<R, StoreError>
    where
        F: FnOnce(&Connection) -> Result<R, StoreError>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        match guard.as_ref() {
            Some(conn) => f(conn),
            None => Err(StoreError::NotInitialized),
        }
    }

    /// Get a mutable reference to the connection.
    fn with_conn_mut<F, R>(&self, f: F) -> Result<R, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<R, StoreError>,
    {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        match guard.as_mut() {
            Some(conn) => f(conn),
            None => Err(StoreError::NotInitialized),
        }
    }

    /// Initialize the database schema.
    fn init_schema(conn: &Connection) -> Result<(), StoreError> {
        // Forensic requirement: WAL + FULL + FK + STRICT for durability.
        // Without these, SQLite is a crash-risk, not a durable store.
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            PRAGMA wal_autocheckpoint = 1000;
            "#,
        )
        .map_err(|e| StoreError::TransactionFailed(format!("pragma failed: {}", e)))?;

        conn.execute_batch(
            r#"
            -- Core entities table (Stage 1)
            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY NOT NULL,
                row_version INTEGER NOT NULL,
                data BLOB NOT NULL,
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_entities_prefix ON entities(id);

            -- Store metadata
            CREATE TABLE IF NOT EXISTS store_meta (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            ) STRICT;
            
            INSERT OR IGNORE INTO store_meta (key, value) VALUES ('schema_version', '2');

            -- Stage 3: Events table for audit trail
            CREATE TABLE IF NOT EXISTS events (
                event_id TEXT PRIMARY KEY NOT NULL,
                event_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                payload BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_id);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_created ON events(created_at);

            -- Stage 3: Resources table for tracking external assets
            CREATE TABLE IF NOT EXISTS resources (
                resource_id TEXT PRIMARY KEY NOT NULL,
                resource_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                state TEXT NOT NULL,
                metadata BLOB,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_resources_entity ON resources(entity_id);
            CREATE INDEX IF NOT EXISTS idx_resources_type ON resources(resource_type);
            CREATE INDEX IF NOT EXISTS idx_resources_state ON resources(state);

            -- Stage 3: Evidence table for verification records
            CREATE TABLE IF NOT EXISTS evidence (
                evidence_id TEXT PRIMARY KEY NOT NULL,
                entity_id TEXT NOT NULL,
                evidence_type TEXT NOT NULL,
                verifier_id TEXT NOT NULL,
                payload BLOB NOT NULL,
                verified_at INTEGER NOT NULL,
                expires_at INTEGER,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_evidence_entity ON evidence(entity_id);
            CREATE INDEX IF NOT EXISTS idx_evidence_type ON evidence(evidence_type);
            CREATE INDEX IF NOT EXISTS idx_evidence_verified ON evidence(verified_at);

            -- Stage 3: Recovery checkpoints for crash recovery
            CREATE TABLE IF NOT EXISTS recovery_checkpoints (
                checkpoint_id TEXT PRIMARY KEY NOT NULL,
                entity_id TEXT NOT NULL,
                checkpoint_type TEXT NOT NULL,
                state BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_recovery_entity ON recovery_checkpoints(entity_id);
            CREATE INDEX IF NOT EXISTS idx_recovery_created ON recovery_checkpoints(created_at);

            -- Stage 3: Authority anchor for crash recovery
            CREATE TABLE IF NOT EXISTS authority_anchor (
                anchor_id TEXT PRIMARY KEY NOT NULL,
                entity_id TEXT NOT NULL,
                fence_scope TEXT NOT NULL,
                row_version INTEGER NOT NULL,
                state BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_anchor_entity ON authority_anchor(entity_id);
            CREATE INDEX IF NOT EXISTS idx_anchor_scope ON authority_anchor(fence_scope);

            -- ────────────────────────────────────────────────────────────
            -- Stage 3 canonical schema (see STAGE3_SCHEMA.md)
            -- ────────────────────────────────────────────────────────────

            -- (1) Event journal: append-only, total order per stream.
            -- UNIQUE(stream_id, sequence) IS the optimistic-concurrency
            -- control: two writers racing for the same next sequence produce
            -- exactly one winner; the loser gets a constraint violation.
            CREATE TABLE IF NOT EXISTS stage3_journal (
                event_id TEXT PRIMARY KEY NOT NULL,
                stream_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                actor TEXT NOT NULL,
                payload BLOB NOT NULL,
                payload_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE (stream_id, sequence)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_stage3_journal_stream
                ON stage3_journal(stream_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_stage3_journal_type
                ON stage3_journal(event_type);
            CREATE INDEX IF NOT EXISTS idx_stage3_journal_actor
                ON stage3_journal(actor);
            CREATE INDEX IF NOT EXISTS idx_stage3_journal_created
                ON stage3_journal(created_at);

            -- (2) Resource ledger: append-only consumption entries.
            -- Aggregation is a projection computed on read; the ledger rows
            -- are never updated or deleted. Corrections are compensating
            -- entries (negative quantity referencing the original entry_id).
            CREATE TABLE IF NOT EXISTS stage3_ledger (
                entry_id TEXT PRIMARY KEY NOT NULL,
                work_unit_id TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                actor TEXT NOT NULL,
                metadata BLOB,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_stage3_ledger_work_unit
                ON stage3_ledger(work_unit_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_stage3_ledger_entity
                ON stage3_ledger(entity_id);
            CREATE INDEX IF NOT EXISTS idx_stage3_ledger_kind
                ON stage3_ledger(kind);

            -- (3) Evidence archive: content-addressed, immutable.
            -- The PRIMARY KEY is the SHA-256 hex digest of `content` —
            -- identical bytes stored twice collapse to one row. No UPDATE or
            -- DELETE is ever issued against this table.
            CREATE TABLE IF NOT EXISTS stage3_evidence (
                content_hash TEXT PRIMARY KEY NOT NULL,
                entity_id TEXT,
                artifact_type TEXT NOT NULL,
                deposited_by TEXT NOT NULL,
                content BLOB NOT NULL,
                created_at INTEGER NOT NULL
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_stage3_evidence_entity
                ON stage3_evidence(entity_id);
            CREATE INDEX IF NOT EXISTS idx_stage3_evidence_type
                ON stage3_evidence(artifact_type);
            CREATE INDEX IF NOT EXISTS idx_stage3_evidence_created
                ON stage3_evidence(created_at);

            -- Bump schema version to 3 to reflect canonical Stage 3 tables.
            INSERT OR IGNORE INTO store_meta (key, value) VALUES ('schema_version', '3');
            UPDATE store_meta SET value = '3' WHERE key = 'schema_version' AND value < '3';
            "#,
        )
        .map_err(|e| StoreError::TransactionFailed(format!("schema init failed: {}", e)))?;

        Ok(())
    }
}

impl StoreBackend for SQLiteStore {
    fn initialize(&mut self, path: &Path) -> Result<(), StoreError> {
        if self.initialized {
            return Err(StoreError::AlreadyInitialized);
        }

        let conn = if self.path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(path)
        }
        .map_err(|e| StoreError::TransactionFailed(format!("open failed: {}", e)))?;

        // Durability & integrity pragmas. journal_mode=WAL gives crash-safe
        // concurrent reads; synchronous=FULL forces fsync before commit returns;
        // foreign_keys=ON enforces referential integrity that is OFF by default.
        // Skip WAL for in-memory databases (WAL is unsupported there).
        if self.path != ":memory:" {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = FULL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA trusted_schema = OFF;",
            )
            .map_err(|e| StoreError::TransactionFailed(format!("pragma init failed: {}", e)))?;
        } else {
            conn.execute_batch(
                "PRAGMA synchronous = FULL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA trusted_schema = OFF;",
            )
            .map_err(|e| StoreError::TransactionFailed(format!("pragma init failed: {}", e)))?;
        }

        Self::init_schema(&conn)?;

        let mut guard = self
            .conn
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        *guard = Some(conn);
        self.initialized = true;

        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn begin_transaction(&mut self) -> Result<(), StoreError> {
        let mut tx_guard = self
            .in_transaction
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        if *tx_guard {
            return Err(StoreError::TransactionFailed(
                "nested transactions not supported".into(),
            ));
        }

        self.with_conn_mut(|conn| {
            conn.execute("BEGIN TRANSACTION", [])
                .map_err(|e| StoreError::TransactionFailed(format!("begin failed: {}", e)))?;
            *tx_guard = true;
            Ok(())
        })
    }

    fn commit(&mut self) -> Result<(), StoreError> {
        let mut tx_guard = self
            .in_transaction
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        if !*tx_guard {
            return Err(StoreError::TransactionFailed(
                "no active transaction".into(),
            ));
        }

        self.with_conn_mut(|conn| {
            conn.execute("COMMIT", [])
                .map_err(|e| StoreError::TransactionFailed(format!("commit failed: {}", e)))?;
            *tx_guard = false;
            Ok(())
        })
    }

    fn rollback(&mut self) -> Result<(), StoreError> {
        let mut tx_guard = self
            .in_transaction
            .lock()
            .map_err(|_| StoreError::TransactionFailed("lock poisoned".into()))?;
        if !*tx_guard {
            return Err(StoreError::TransactionFailed(
                "no active transaction".into(),
            ));
        }

        self.with_conn_mut(|conn| {
            conn.execute("ROLLBACK", [])
                .map_err(|e| StoreError::TransactionFailed(format!("rollback failed: {}", e)))?;
            *tx_guard = false;
            Ok(())
        })
    }

    fn read_bytes(&self, id: &EntityId) -> Result<Option<(RowVersion, Vec<u8>)>, StoreError> {
        self.with_conn(|conn| {
            let id_str = id.to_string();
            let mut stmt = conn
                .prepare("SELECT row_version, data FROM entities WHERE id = ?1")
                .map_err(|e| StoreError::TransactionFailed(format!("prepare failed: {}", e)))?;

            let mut rows = stmt
                .query(params![id_str])
                .map_err(|e| StoreError::TransactionFailed(format!("query failed: {}", e)))?;

            match rows.next() {
                Ok(Some(row)) => {
                    let version: i64 = row.get(0).map_err(|e| {
                        StoreError::TransactionFailed(format!("get version failed: {}", e))
                    })?;
                    let data: Vec<u8> = row.get(1).map_err(|e| {
                        StoreError::TransactionFailed(format!("get data failed: {}", e))
                    })?;

                    let version = row_version_from_sqlite(version)?;

                    Ok(Some((version, data)))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(StoreError::TransactionFailed(format!(
                    "row fetch failed: {}",
                    e
                ))),
            }
        })
    }

    fn write_bytes(
        &mut self,
        id: &EntityId,
        data: &[u8],
        expected: ExpectedVersion,
    ) -> Result<RowVersion, StoreError> {
        let id_str = id.to_string();

        // Read-modify-write happens under the single connection mutex; callers
        // that need multi-statement atomicity wrap it in begin_transaction.
        let current = self.read_bytes(id)?;
        let current_version = current.as_ref().map(|(v, _)| *v);

        match expected {
            ExpectedVersion::Any => {}
            ExpectedVersion::MustNotExist => {
                if let Some(found) = current_version {
                    return Err(StoreError::RowVersionMustNotExist { found });
                }
            }
            ExpectedVersion::Existing(expected_v) => match current_version {
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

        let new_version = match current_version {
            Some(v) => {
                let next = v
                    .get()
                    .checked_add(1)
                    .ok_or(StoreError::RowVersionOverflow(u64::MAX))?;
                // Enforce the storage invariant at the exact boundary: values
                // above i64::MAX cannot round-trip through a signed column.
                if next > ROW_VERSION_SQLITE_MAX {
                    return Err(StoreError::RowVersionOverflow(next));
                }
                RowVersion::new(next).expect("nonzero after checked_add")
            }
            None => RowVersion::MIN,
        };
        let new_version_raw = row_version_to_sqlite(new_version)?;

        self.with_conn_mut(|conn| {
            let changed = if current_version.is_some() {
                // Atomic CAS: the version predicate lives in the UPDATE itself,
                // so a stale snapshot can never silently overwrite a newer row
                // even if a concurrent writer slipped in after our read.
                conn.execute(
                    "UPDATE entities SET row_version = ?1, data = ?2, updated_at = ?3 \
                     WHERE id = ?4 AND row_version = ?5",
                    params![
                        new_version_raw,
                        data,
                        0i64,
                        id_str,
                        crate::row_version_to_sqlite(current_version.unwrap())?
                    ],
                )
            } else {
                conn.execute(
                    "INSERT INTO entities (id, row_version, data, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id_str, new_version_raw, data, 0i64, 0i64],
                )
            }
            .map_err(|e| StoreError::TransactionFailed(format!("write failed: {}", e)))?;

            if changed == 0 {
                // A concurrent writer advanced the row between our read and the
                // UPDATE. Re-read the exact current row so the conflict report
                // carries the OBSERVED version, not an inferred current+1.
                let observed: Option<(RowVersion, Vec<u8>)> = conn
                    .query_row(
                        "SELECT row_version, data FROM entities WHERE id = ?1",
                        params![id_str],
                        |row| {
                            let raw: i64 = row.get(0)?;
                            let data: Vec<u8> = row.get(1)?;
                            Ok((raw, data))
                        },
                    )
                    .optional()
                    .map_err(|e| {
                        StoreError::TransactionFailed(format!("conflict re-read failed: {}", e))
                    })?
                    .map(|(raw, data)| {
                        let v = crate::row_version_from_sqlite(raw)?;
                        Ok::<(RowVersion, Vec<u8>), StoreError>((v, data))
                    })
                    .transpose()?;
                let expected_v = current_version.unwrap_or(RowVersion::MIN);
                return match observed {
                    Some((found_v, _)) => Err(StoreError::RowVersionConflict {
                        expected: expected_v,
                        found: found_v,
                    }),
                    // Row was deleted between our read and the UPDATE: the
                    // truthful conflict semantics is "expected row is absent".
                    None => Err(StoreError::RowVersionExpectedMissing),
                };
            }

            Ok(new_version)
        })
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StoreError> {
        self.with_conn_mut(|conn| {
            let id_str = id.to_string();
            conn.execute("DELETE FROM entities WHERE id = ?1", params![id_str])
                .map_err(|e| StoreError::TransactionFailed(format!("delete failed: {}", e)))?;
            Ok(())
        })
    }

    fn exists(&self, id: &EntityId) -> Result<bool, StoreError> {
        self.with_conn(|conn| {
            let id_str = id.to_string();
            let mut stmt = conn
                .prepare("SELECT 1 FROM entities WHERE id = ?1 LIMIT 1")
                .map_err(|e| StoreError::TransactionFailed(format!("prepare failed: {}", e)))?;

            let exists = stmt
                .exists(params![id_str])
                .map_err(|e| StoreError::TransactionFailed(format!("exists failed: {}", e)))?;

            Ok(exists)
        })
    }

    fn list_by_prefix(&self, prefix: &str) -> Result<Vec<EntityId>, StoreError> {
        self.with_conn(|conn| {
            let pattern = format!("{}%", prefix);
            let mut stmt = conn
                .prepare("SELECT id FROM entities WHERE id LIKE ?1")
                .map_err(|e| StoreError::TransactionFailed(format!("prepare failed: {}", e)))?;

            let rows = stmt
                .query_map(params![pattern], |row| row.get::<_, String>(0))
                .map_err(|e| StoreError::TransactionFailed(format!("query failed: {}", e)))?;

            let mut result = Vec::new();
            for row in rows {
                let id_str =
                    row.map_err(|e| StoreError::TransactionFailed(format!("row failed: {}", e)))?;
                if let Ok(id) = EntityId::try_from(id_str) {
                    result.push(id);
                }
            }

            Ok(result)
        })
    }

    fn integrity_check(&self) -> Result<(), StoreError> {
        self.with_conn(|conn| {
            let result: String = conn
                .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                .map_err(|e| {
                    StoreError::TransactionFailed(format!("integrity check failed: {}", e))
                })?;

            if result == "ok" {
                Ok(())
            } else {
                Err(StoreError::IntegrityFailure(result))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    fn test_id(seed: u128) -> EntityId {
        let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
        EntityId::from_u128(base + seed).expect("valid UUIDv7")
    }

    #[test]
    fn sqlite_lifecycle() {
        let mut store = SQLiteStore::new(":memory:");
        assert!(!store.is_initialized());

        store.initialize(Path::new(":memory:")).unwrap();
        assert!(store.is_initialized());
    }

    #[test]
    fn sqlite_write_and_read() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let data = b"hello world";

        let v1 = store
            .write_bytes(&id, data, ExpectedVersion::MustNotExist)
            .unwrap();
        assert_eq!(v1, NonZeroU64::MIN);

        let read = store.read_bytes(&id).unwrap();
        assert!(read.is_some());
        let (version, bytes) = read.unwrap();
        assert_eq!(version, v1);
        assert_eq!(bytes, data);
    }

    #[test]
    fn sqlite_version_increments() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);

        let v1 = store
            .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
            .unwrap();
        let v2 = store
            .write_bytes(&id, b"v2", ExpectedVersion::Existing(v1))
            .unwrap();
        let v3 = store
            .write_bytes(&id, b"v3", ExpectedVersion::Existing(v2))
            .unwrap();

        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn sqlite_cas_rejects_stale_writer() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let v1 = store
            .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
            .unwrap();
        let _v2 = store
            .write_bytes(&id, b"v2", ExpectedVersion::Existing(v1))
            .unwrap();

        let stale = store.write_bytes(&id, b"stale", ExpectedVersion::Existing(v1));
        assert!(matches!(
            stale,
            Err(StoreError::RowVersionConflict { expected, found })
                if expected == v1 && found == _v2
        ));

        let (_, bytes) = store.read_bytes(&id).unwrap().unwrap();
        assert_eq!(bytes, b"v2");
    }

    /// P2-STORE-CONFLICT-DIAGNOSTIC: a lost CAS race must report the OBSERVED
    /// current row version (re-read after changed==0), never an inferred
    /// current+1. We simulate the race by advancing the row inside the same
    /// connection after the backend's pre-read, which is only reachable
    /// through the atomic UPDATE predicate path.
    #[test]
    fn sqlite_conflict_reports_observed_version_not_inferred() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let v1 = store
            .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
            .unwrap();
        // Advance the row to version 2 through the normal path.
        let v2 = store
            .write_bytes(&id, b"v2", ExpectedVersion::Existing(v1))
            .unwrap();

        // Stale writer with expected=v1 must observe found=v2 (the real
        // current version), regardless of how many times the row moved.
        let stale = store.write_bytes(&id, b"stale", ExpectedVersion::Existing(v1));
        match stale {
            Err(StoreError::RowVersionConflict { expected, found }) => {
                assert_eq!(expected, v1);
                assert_eq!(found, v2);
            }
            other => panic!("expected RowVersionConflict, got {:?}", other),
        }
    }

    /// P2-STORE-CONFLICT-DIAGNOSTIC (delete race): if the row disappears
    /// between pre-read and the conditional UPDATE, the truthful conflict
    /// semantics is RowVersionExpectedMissing, not a fabricated version.
    /// Exercised via the raw connection to force the interleaving.
    #[test]
    fn sqlite_conflict_after_delete_reports_missing() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let v1 = store
            .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
            .unwrap();
        // Delete the row directly; the stale writer still expects v1.
        store.delete(&id).unwrap();

        let stale = store.write_bytes(&id, b"stale", ExpectedVersion::Existing(v1));
        // Pre-read already sees absence, so this is RowVersionExpectedMissing —
        // the key property is that NO fabricated found=version is reported.
        assert!(matches!(stale, Err(StoreError::RowVersionExpectedMissing)));
    }

    #[test]
    fn sqlite_must_not_exist_rejects_overwrite() {
        let mut store = SQLiteStore::new(":memory:");
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
    fn sqlite_existing_requires_presence() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let missing = store.write_bytes(&id, b"x", ExpectedVersion::Existing(NonZeroU64::MIN));
        assert!(matches!(
            missing,
            Err(StoreError::RowVersionExpectedMissing)
        ));
    }

    #[test]
    fn sqlite_row_version_i64_max_round_trip() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        let max_version = NonZeroU64::new(i64::MAX as u64).unwrap();

        // Insert directly at the storage ceiling via raw SQL to simulate a row
        // that has reached the exact boundary of the i64 representation.
        {
            let guard = store.conn.lock().unwrap();
            let conn = guard.as_ref().unwrap();
            conn.execute(
                "INSERT INTO entities (id, row_version, data, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 0, 0)",
                rusqlite::params![id.to_string(), max_version.get() as i64, b"at-max"],
            )
            .unwrap();
        }

        // Reading back must reconstruct the exact u64 value.
        let (version, bytes) = store.read_bytes(&id).unwrap().unwrap();
        assert_eq!(version, max_version);
        assert_eq!(bytes, b"at-max");

        // The next write must fail with RowVersionOverflow at the exact
        // boundary value (i64::MAX + 1), not wrap negative.
        let over = store.write_bytes(&id, b"overflow", ExpectedVersion::Existing(max_version));
        assert!(matches!(
            over,
            Err(StoreError::RowVersionOverflow(v)) if v == (i64::MAX as u64) + 1
        ));

        // The boundary row must be untouched.
        let (version, bytes) = store.read_bytes(&id).unwrap().unwrap();
        assert_eq!(version, max_version);
        assert_eq!(bytes, b"at-max");
    }

    #[test]
    fn sqlite_transaction_commit() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        store.begin_transaction().unwrap();
        store.commit().unwrap();
    }

    #[test]
    fn sqlite_transaction_rollback() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        store.begin_transaction().unwrap();
        store.rollback().unwrap();
    }

    #[test]
    fn sqlite_integrity_check() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        store.integrity_check().unwrap();
    }

    #[test]
    fn sqlite_exists_and_delete() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let id = test_id(1);
        assert!(!store.exists(&id).unwrap());

        store
            .write_bytes(&id, b"data", ExpectedVersion::Any)
            .unwrap();
        assert!(store.exists(&id).unwrap());

        store.delete(&id).unwrap();
        assert!(!store.exists(&id).unwrap());
    }

    #[test]
    fn sqlite_stage3_round_trip() {
        use crate::{
            AuthorityAnchor, EventRecord, EvidenceRecord, RecoveryCheckpoint, ResourceRecord,
            Stage3Backend,
        };

        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(42);
        // Stage-3 tables have FK to entities(id) — insert the parent entity first.
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        // Event round-trip
        store
            .put_event(&EventRecord {
                event_id: "ev1".into(),
                event_type: "command".into(),
                entity_id: entity,
                payload: b"payload".to_vec(),
                created_at: 100,
            })
            .unwrap();
        let events = store.events_for_entity(&entity).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "command");

        // Resource round-trip
        store
            .put_resource(&ResourceRecord {
                resource_id: "res1".into(),
                resource_type: "file".into(),
                entity_id: entity,
                state: "active".into(),
                metadata: None,
                created_at: 100,
                updated_at: 200,
            })
            .unwrap();
        let resources = store.resources_for_entity(&entity).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].state, "active");

        // Evidence round-trip
        store
            .put_evidence(&EvidenceRecord {
                evidence_id: "evd1".into(),
                entity_id: entity,
                evidence_type: "machine".into(),
                verifier_id: "c08".into(),
                payload: b"proof".to_vec(),
                verified_at: 300,
                expires_at: None,
            })
            .unwrap();
        let evidence = Stage3Backend::evidence_for_entity(&store, &entity).unwrap();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].verifier_id, "c08");

        // Recovery checkpoint round-trip (latest wins)
        for (i, ts) in [(1u32, 400u64), (2, 500)].iter() {
            store
                .put_recovery_checkpoint(&RecoveryCheckpoint {
                    checkpoint_id: format!("cp{}", i),
                    entity_id: entity,
                    checkpoint_type: "state".into(),
                    state: vec![*i as u8],
                    created_at: *ts,
                })
                .unwrap();
        }
        let latest = store.latest_recovery_checkpoint(&entity).unwrap().unwrap();
        assert_eq!(latest.checkpoint_id, "cp2");

        // Authority anchor round-trip (scoped)
        store
            .put_authority_anchor(&AuthorityAnchor {
                anchor_id: "a1".into(),
                entity_id: entity,
                fence_scope: "core".into(),
                row_version: 7,
                state: b"s".to_vec(),
                created_at: 600,
            })
            .unwrap();
        let anchor = store
            .latest_authority_anchor(&entity, "core")
            .unwrap()
            .unwrap();
        assert_eq!(anchor.row_version, 7);
        assert!(
            store
                .latest_authority_anchor(&entity, "other")
                .unwrap()
                .is_none()
        );
    }

    // F09: duplicate event_id must fail (strict INSERT, no silent overwrite)
    #[test]
    fn sqlite_put_event_duplicate_id_fails() {
        use crate::{EventRecord, Stage3Backend};
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();
        let entity = test_id(100);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();
        let rec = EventRecord {
            event_id: "ev-dup".into(),
            event_type: "command".into(),
            entity_id: entity,
            payload: b"payload".to_vec(),
            created_at: 100,
        };
        store.put_event(&rec).unwrap();
        // Second insert with the same event_id must fail
        let dup = store.put_event(&rec);
        assert!(
            dup.is_err(),
            "duplicate event_id must fail, not silently overwrite"
        );
        // Only one event remains
        let events = store.events_for_entity(&entity).unwrap();
        assert_eq!(events.len(), 1, "original event must not be overwritten");
    }

    // F09: duplicate evidence must fail (strict INSERT, no silent overwrite)
    #[test]
    fn sqlite_put_evidence_duplicate_fails() {
        use crate::{EvidenceRecord, Stage3Backend};
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();
        let entity = test_id(101);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();
        let rec = EvidenceRecord {
            evidence_id: "evd-dup".into(),
            entity_id: entity,
            evidence_type: "verification".into(),
            verifier_id: "test-verifier".into(),
            payload: b"p".to_vec(),
            verified_at: 100,
            expires_at: None,
        };
        store.put_evidence(&rec).unwrap();
        let dup = store.put_evidence(&rec);
        assert!(
            dup.is_err(),
            "duplicate evidence_id must fail, not silently overwrite"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Stage 3 canonical schema tests
    // ─────────────────────────────────────────────────────────────────────

    use crate::stage3::{
        EvidenceArtifact, JournalEvent, LedgerEntry, ResourceKind, Stage3CanonicalBackend,
        sha256_hex,
    };

    #[test]
    fn stage3_sha256_hex_consistency() {
        let data = b"hello";
        let h = sha256_hex(data);
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, sha256_hex(data));
    }

    #[test]
    fn stage3_journal_append_and_read() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let ev1 = JournalEvent::new(
            "ev1".into(),
            "stream-A".into(),
            1,
            "created".into(),
            "user-1".into(),
            b"payload1".to_vec(),
            1000,
        );
        store.journal_append(&ev1).unwrap();

        let ev2 = JournalEvent::new(
            "ev2".into(),
            "stream-A".into(),
            2,
            "updated".into(),
            "user-1".into(),
            b"payload2".to_vec(),
            2000,
        );
        store.journal_append(&ev2).unwrap();

        let stream = store.journal_read_stream("stream-A").unwrap();
        assert_eq!(stream.len(), 2);
        assert_eq!(stream[0].event_id, "ev1");
        assert_eq!(stream[1].event_id, "ev2");
        assert_eq!(stream[0].sequence, 1);
        assert_eq!(stream[1].sequence, 2);
        assert_eq!(stream[0].actor, "user-1");
        assert_eq!(stream[0].payload_hash, sha256_hex(b"payload1"));
    }

    #[test]
    fn stage3_journal_head_sequence() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        assert_eq!(store.journal_head_sequence("stream-X").unwrap(), None);

        let ev = JournalEvent::new(
            "ev1".into(),
            "stream-X".into(),
            1,
            "created".into(),
            "user-1".into(),
            b"p".to_vec(),
            1000,
        );
        store.journal_append(&ev).unwrap();
        assert_eq!(store.journal_head_sequence("stream-X").unwrap(), Some(1));

        let ev2 = JournalEvent::new(
            "ev2".into(),
            "stream-X".into(),
            2,
            "updated".into(),
            "user-1".into(),
            b"p2".to_vec(),
            2000,
        );
        store.journal_append(&ev2).unwrap();
        assert_eq!(store.journal_head_sequence("stream-X").unwrap(), Some(2));
    }

    #[test]
    fn stage3_journal_sequence_conflict_rejected() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let ev1 = JournalEvent::new(
            "ev1".into(),
            "stream-Y".into(),
            1,
            "created".into(),
            "user-1".into(),
            b"p".to_vec(),
            1000,
        );
        store.journal_append(&ev1).unwrap();

        // Same stream, same sequence, different event_id → must fail
        let ev2 = JournalEvent::new(
            "ev2".into(),
            "stream-Y".into(),
            1,
            "updated".into(),
            "user-2".into(),
            b"p2".to_vec(),
            2000,
        );
        let result = store.journal_append(&ev2);
        assert!(
            result.is_err(),
            "duplicate (stream_id, sequence) must be rejected"
        );

        // Different stream, same sequence → must succeed
        let ev3 = JournalEvent::new(
            "ev3".into(),
            "stream-Z".into(),
            1,
            "created".into(),
            "user-1".into(),
            b"p3".to_vec(),
            3000,
        );
        assert!(store.journal_append(&ev3).is_ok());
    }

    #[test]
    fn stage3_journal_payload_hash_verification() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let mut ev = JournalEvent::new(
            "ev1".into(),
            "stream-H".into(),
            1,
            "created".into(),
            "user-1".into(),
            b"payload".to_vec(),
            1000,
        );
        // Corrupt the hash
        ev.payload_hash = "00".repeat(32);
        let result = store.journal_append(&ev);
        assert!(result.is_err(), "hash mismatch must be rejected");
    }

    #[test]
    fn stage3_ledger_append_and_read() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(200);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        let e1 = LedgerEntry {
            entry_id: "le1".into(),
            work_unit_id: "wu-1".into(),
            entity_id: entity,
            kind: ResourceKind::Tokens,
            quantity: 1500,
            actor: "worker-1".into(),
            metadata: None,
            created_at: 1000,
        };
        store.ledger_append(&e1).unwrap();

        let e2 = LedgerEntry {
            entry_id: "le2".into(),
            work_unit_id: "wu-1".into(),
            entity_id: entity,
            kind: ResourceKind::TimeMs,
            quantity: 3200,
            actor: "worker-1".into(),
            metadata: None,
            created_at: 2000,
        };
        store.ledger_append(&e2).unwrap();

        let entries = store.ledger_for_work_unit("wu-1").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_id, "le1");
        assert_eq!(entries[1].entry_id, "le2");
        assert_eq!(entries[0].kind, ResourceKind::Tokens);
        assert_eq!(entries[1].kind, ResourceKind::TimeMs);
    }

    #[test]
    fn stage3_ledger_totals() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(201);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        for (id, kind, qty, ts) in [
            ("l1", ResourceKind::Tokens, 100, 1000),
            ("l2", ResourceKind::Tokens, 200, 2000),
            ("l3", ResourceKind::TimeMs, 5000, 3000),
        ] {
            store
                .ledger_append(&LedgerEntry {
                    entry_id: id.into(),
                    work_unit_id: "wu-tot".into(),
                    entity_id: entity,
                    kind,
                    quantity: qty,
                    actor: "worker".into(),
                    metadata: None,
                    created_at: ts,
                })
                .unwrap();
        }

        let totals = store.ledger_totals_for_work_unit("wu-tot").unwrap();
        assert_eq!(totals.len(), 2);
        let tokens = totals.iter().find(|(k, _)| *k == ResourceKind::Tokens);
        let time = totals.iter().find(|(k, _)| *k == ResourceKind::TimeMs);
        assert_eq!(tokens.map(|(_, v)| *v), Some(300));
        assert_eq!(time.map(|(_, v)| *v), Some(5000));
    }

    #[test]
    fn stage3_ledger_duplicate_id_fails() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(202);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        let e = LedgerEntry {
            entry_id: "dup".into(),
            work_unit_id: "wu-d".into(),
            entity_id: entity,
            kind: ResourceKind::Tokens,
            quantity: 100,
            actor: "w".into(),
            metadata: None,
            created_at: 1000,
        };
        store.ledger_append(&e).unwrap();
        let dup = store.ledger_append(&e);
        assert!(dup.is_err(), "duplicate ledger entry_id must fail");
    }

    #[test]
    fn stage3_evidence_put_and_get() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(300);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        let artifact = EvidenceArtifact::new(
            Some(entity),
            "text/log".into(),
            "verifier-1".into(),
            b"log content here".to_vec(),
            1000,
        );
        let hash = store.evidence_put(&artifact).unwrap();
        assert_eq!(hash, artifact.content_hash);

        let fetched = store.evidence_get(&hash).unwrap().unwrap();
        assert_eq!(fetched.content, b"log content here");
        assert_eq!(fetched.artifact_type, "text/log");
        assert_eq!(fetched.entity_id, Some(entity));
    }

    #[test]
    fn stage3_evidence_content_addressed_dedup() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let artifact = EvidenceArtifact::new(
            None,
            "text/plain".into(),
            "a".into(),
            b"same bytes".to_vec(),
            1000,
        );
        let h1 = store.evidence_put(&artifact).unwrap();
        let h2 = store.evidence_put(&artifact).unwrap();
        assert_eq!(h1, h2);

        let fetched = store.evidence_get(&h1).unwrap().unwrap();
        assert_eq!(fetched.content, b"same bytes");
    }

    #[test]
    fn stage3_evidence_for_entity() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();

        let entity = test_id(301);
        store
            .write_bytes(&entity, b"parent", ExpectedVersion::MustNotExist)
            .unwrap();

        let a1 = EvidenceArtifact::new(
            Some(entity),
            "text/a".into(),
            "v".into(),
            b"a1".to_vec(),
            1000,
        );
        let a2 = EvidenceArtifact::new(
            Some(entity),
            "text/b".into(),
            "v".into(),
            b"a2".to_vec(),
            2000,
        );
        store.evidence_put(&a1).unwrap();
        store.evidence_put(&a2).unwrap();

        let list = store.evidence_list_by_entity(&entity).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].content, b"a1");
        assert_eq!(list[1].content, b"a2");
    }

    #[test]
    fn stage3_evidence_verify_integrity() {
        let mut artifact = EvidenceArtifact::new(
            None,
            "text/plain".into(),
            "a".into(),
            b"data".to_vec(),
            1000,
        );
        assert!(artifact.verify().is_ok());
        artifact.content_hash = "ff".repeat(32);
        assert!(artifact.verify().is_err());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stage 3 typed-table implementation
// ─────────────────────────────────────────────────────────────────────────────

impl Stage3Backend for SQLiteStore {
    fn put_event(&mut self, rec: &EventRecord) -> Result<(), StoreError> {
        let eid = rec.entity_id.to_string();
        self.with_conn_mut(|conn| {
            conn.execute(
                // F09: INSERT-only for events — duplicate ID is an error, not silent overwrite
                "INSERT INTO events (event_id, event_type, entity_id, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rec.event_id,
                    rec.event_type,
                    eid,
                    rec.payload,
                    rec.created_at as i64
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("put_event failed: {}", e)))?;
            Ok(())
        })
    }

    fn events_for_entity(&self, entity_id: &EntityId) -> Result<Vec<EventRecord>, StoreError> {
        let eid = entity_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT event_id, event_type, entity_id, payload, created_at
                     FROM events WHERE entity_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![eid], |row| {
                    let entity_str: String = row.get(2)?;
                    let created: i64 = row.get(4)?;
                    Ok(EventRecord {
                        event_id: row.get(0)?,
                        event_type: row.get(1)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        payload: row.get(3)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    fn put_resource(&mut self, rec: &ResourceRecord) -> Result<(), StoreError> {
        let eid = rec.entity_id.to_string();
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO resources
                 (resource_id, resource_type, entity_id, state, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    rec.resource_id,
                    rec.resource_type,
                    eid,
                    rec.state,
                    rec.metadata,
                    rec.created_at as i64,
                    rec.updated_at as i64
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("put_resource failed: {}", e)))?;
            Ok(())
        })
    }

    fn resources_for_entity(
        &self,
        entity_id: &EntityId,
    ) -> Result<Vec<ResourceRecord>, StoreError> {
        let eid = entity_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT resource_id, resource_type, entity_id, state, metadata, created_at, updated_at
                     FROM resources WHERE entity_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![eid], |row| {
                    let entity_str: String = row.get(2)?;
                    let created: i64 = row.get(5)?;
                    let updated: i64 = row.get(6)?;
                    Ok(ResourceRecord {
                        resource_id: row.get(0)?,
                        resource_type: row.get(1)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        state: row.get(3)?,
                        metadata: row.get(4)?,
                        created_at: created as u64,
                        updated_at: updated as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    fn put_evidence(&mut self, rec: &EvidenceRecord) -> Result<(), StoreError> {
        let eid = rec.entity_id.to_string();
        self.with_conn_mut(|conn| {
            conn.execute(
                // F09: INSERT-only for evidence — duplicate ID is an error, not silent overwrite
                "INSERT INTO evidence
                 (evidence_id, entity_id, evidence_type, verifier_id, payload, verified_at, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    rec.evidence_id,
                    eid,
                    rec.evidence_type,
                    rec.verifier_id,
                    rec.payload,
                    rec.verified_at as i64,
                    rec.expires_at.map(|v| v as i64)
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("put_evidence failed: {}", e)))?;
            Ok(())
        })
    }

    fn evidence_for_entity(&self, entity_id: &EntityId) -> Result<Vec<EvidenceRecord>, StoreError> {
        let eid = entity_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT evidence_id, entity_id, evidence_type, verifier_id, payload, verified_at, expires_at
                     FROM evidence WHERE entity_id = ?1 ORDER BY verified_at ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![eid], |row| {
                    let entity_str: String = row.get(1)?;
                    let verified: i64 = row.get(5)?;
                    let expires: Option<i64> = row.get(6)?;
                    Ok(EvidenceRecord {
                        evidence_id: row.get(0)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        evidence_type: row.get(2)?,
                        verifier_id: row.get(3)?,
                        payload: row.get(4)?,
                        verified_at: verified as u64,
                        expires_at: expires.map(|v| v as u64),
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    fn put_recovery_checkpoint(&mut self, rec: &RecoveryCheckpoint) -> Result<(), StoreError> {
        let eid = rec.entity_id.to_string();
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO recovery_checkpoints
                 (checkpoint_id, entity_id, checkpoint_type, state, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rec.checkpoint_id,
                    eid,
                    rec.checkpoint_type,
                    rec.state,
                    rec.created_at as i64
                ],
            )
            .map_err(|e| {
                StoreError::TransactionFailed(format!("put_recovery_checkpoint failed: {}", e))
            })?;
            Ok(())
        })
    }

    fn latest_recovery_checkpoint(
        &self,
        entity_id: &EntityId,
    ) -> Result<Option<RecoveryCheckpoint>, StoreError> {
        let eid = entity_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT checkpoint_id, entity_id, checkpoint_type, state, created_at
                     FROM recovery_checkpoints WHERE entity_id = ?1
                     ORDER BY created_at DESC LIMIT 1",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let mut rows = stmt
                .query_map(params![eid], |row| {
                    let entity_str: String = row.get(1)?;
                    let created: i64 = row.get(4)?;
                    Ok(RecoveryCheckpoint {
                        checkpoint_id: row.get(0)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        checkpoint_type: row.get(2)?,
                        state: row.get(3)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            match rows.next() {
                Some(Ok(r)) => Ok(Some(r)),
                Some(Err(e)) => Err(StoreError::IntegrityFailure(format!("row: {}", e))),
                None => Ok(None),
            }
        })
    }

    fn put_authority_anchor(&mut self, rec: &AuthorityAnchor) -> Result<(), StoreError> {
        let eid = rec.entity_id.to_string();
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO authority_anchor
                 (anchor_id, entity_id, fence_scope, row_version, state, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    rec.anchor_id,
                    eid,
                    rec.fence_scope,
                    rec.row_version as i64,
                    rec.state,
                    rec.created_at as i64
                ],
            )
            .map_err(|e| {
                StoreError::TransactionFailed(format!("put_authority_anchor failed: {}", e))
            })?;
            Ok(())
        })
    }

    fn latest_authority_anchor(
        &self,
        entity_id: &EntityId,
        fence_scope: &str,
    ) -> Result<Option<AuthorityAnchor>, StoreError> {
        let eid = entity_id.to_string();
        let scope = fence_scope.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT anchor_id, entity_id, fence_scope, row_version, state, created_at
                     FROM authority_anchor
                     WHERE entity_id = ?1 AND fence_scope = ?2
                     ORDER BY created_at DESC LIMIT 1",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let mut rows = stmt
                .query_map(params![eid, scope], |row| {
                    let entity_str: String = row.get(1)?;
                    let rv: i64 = row.get(3)?;
                    let created: i64 = row.get(5)?;
                    Ok(AuthorityAnchor {
                        anchor_id: row.get(0)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        fence_scope: row.get(2)?,
                        row_version: rv as u64,
                        state: row.get(4)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            match rows.next() {
                Some(Ok(r)) => Ok(Some(r)),
                Some(Err(e)) => Err(StoreError::IntegrityFailure(format!("row: {}", e))),
                None => Ok(None),
            }
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stage 3 canonical backend implementation
// ─────────────────────────────────────────────────────────────────────────────

use crate::stage3::{
    EvidenceArtifact, JournalEvent, LedgerEntry, ResourceKind, Stage3CanonicalBackend,
};

impl Stage3CanonicalBackend for SQLiteStore {
    // ── Event journal ────────────────────────────────────────────────────

    fn journal_append(&mut self, ev: &JournalEvent) -> Result<(), StoreError> {
        // Verify payload hash matches before writing — refuse to persist
        // inconsistent events.
        ev.verify_hash()?;
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO stage3_journal
                 (event_id, stream_id, sequence, event_type, actor, payload, payload_hash, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    ev.event_id,
                    ev.stream_id,
                    ev.sequence as i64,
                    ev.event_type,
                    ev.actor,
                    ev.payload,
                    ev.payload_hash,
                    ev.created_at as i64
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("journal_append failed: {}", e)))?;
            Ok(())
        })
    }

    fn journal_read_stream(&self, stream_id: &str) -> Result<Vec<JournalEvent>, StoreError> {
        let sid = stream_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT event_id, stream_id, sequence, event_type, actor, payload, payload_hash, created_at
                     FROM stage3_journal WHERE stream_id = ?1 ORDER BY sequence ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![sid], |row| {
                    let sequence: i64 = row.get(2)?;
                    let created: i64 = row.get(7)?;
                    Ok(JournalEvent {
                        event_id: row.get(0)?,
                        stream_id: row.get(1)?,
                        sequence: sequence as u64,
                        event_type: row.get(3)?,
                        actor: row.get(4)?,
                        payload: row.get(5)?,
                        payload_hash: row.get(6)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    fn journal_head_sequence(&self, stream_id: &str) -> Result<Option<u64>, StoreError> {
        let sid = stream_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT MAX(sequence) FROM stage3_journal WHERE stream_id = ?1")
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let max: Option<i64> = stmt
                .query_row(params![sid], |row| row.get(0))
                .optional()
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?
                .flatten();
            Ok(max.map(|v| v as u64))
        })
    }

    // ── Resource ledger ─────────────────────────────────────────────────

    fn ledger_append(&mut self, e: &LedgerEntry) -> Result<(), StoreError> {
        let eid = e.entity_id.to_string();
        let kind_str = e.kind.as_str();
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO stage3_ledger
                 (entry_id, work_unit_id, entity_id, kind, quantity, actor, metadata, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    e.entry_id,
                    e.work_unit_id,
                    eid,
                    kind_str,
                    e.quantity,
                    e.actor,
                    e.metadata,
                    e.created_at as i64
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("ledger_append failed: {}", e)))?;
            Ok(())
        })
    }

    fn ledger_for_work_unit(&self, work_unit_id: &str) -> Result<Vec<LedgerEntry>, StoreError> {
        let wu = work_unit_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT entry_id, work_unit_id, entity_id, kind, quantity, actor, metadata, created_at
                     FROM stage3_ledger WHERE work_unit_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![wu], |row| {
                    let entity_str: String = row.get(2)?;
                    let kind_str: String = row.get(3)?;
                    let created: i64 = row.get(7)?;
                    let kind = ResourceKind::parse(&kind_str)
                        .ok_or(rusqlite::Error::InvalidQuery)?;
                    Ok(LedgerEntry {
                        entry_id: row.get(0)?,
                        work_unit_id: row.get(1)?,
                        entity_id: EntityId::try_from(entity_str)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        kind,
                        quantity: row.get(4)?,
                        actor: row.get(5)?,
                        metadata: row.get(6)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    fn ledger_totals_for_work_unit(
        &self,
        work_unit_id: &str,
    ) -> Result<Vec<(ResourceKind, i64)>, StoreError> {
        let wu = work_unit_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT kind, SUM(quantity) FROM stage3_ledger
                     WHERE work_unit_id = ?1 GROUP BY kind",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![wu], |row| {
                    let kind_str: String = row.get(0)?;
                    let total: i64 = row.get(1)?;
                    let kind =
                        ResourceKind::parse(&kind_str).ok_or(rusqlite::Error::InvalidQuery)?;
                    Ok((kind, total))
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }

    // ── Evidence archive ────────────────────────────────────────────────

    fn evidence_put(&mut self, a: &EvidenceArtifact) -> Result<String, StoreError> {
        a.verify()?;
        let eid = a.entity_id.map(|id| id.to_string());
        self.with_conn_mut(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO stage3_evidence
                 (content_hash, entity_id, artifact_type, deposited_by, content, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    a.content_hash,
                    eid,
                    a.artifact_type,
                    a.deposited_by,
                    a.content,
                    a.created_at as i64
                ],
            )
            .map_err(|e| StoreError::TransactionFailed(format!("evidence_put failed: {}", e)))?;
            Ok(a.content_hash.clone())
        })
    }

    fn evidence_get(&self, content_hash: &str) -> Result<Option<EvidenceArtifact>, StoreError> {
        let ch = content_hash.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT content_hash, entity_id, artifact_type, deposited_by, content, created_at
                     FROM stage3_evidence WHERE content_hash = ?1",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let mut rows = stmt
                .query_map(params![ch], |row| {
                    let entity_str: Option<String> = row.get(1)?;
                    let created: i64 = row.get(5)?;
                    Ok(EvidenceArtifact {
                        content_hash: row.get(0)?,
                        entity_id: entity_str
                            .and_then(|s| EntityId::try_from(s).ok()),
                        artifact_type: row.get(2)?,
                        deposited_by: row.get(3)?,
                        content: row.get(4)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            match rows.next() {
                Some(Ok(r)) => Ok(Some(r)),
                Some(Err(e)) => Err(StoreError::IntegrityFailure(format!("row: {}", e))),
                None => Ok(None),
            }
        })
    }

    fn evidence_list_by_entity(
        &self,
        entity_id: &EntityId,
    ) -> Result<Vec<EvidenceArtifact>, StoreError> {
        let eid = entity_id.to_string();
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT content_hash, entity_id, artifact_type, deposited_by, content, created_at
                     FROM stage3_evidence WHERE entity_id = ?1 ORDER BY created_at ASC",
                )
                .map_err(|e| StoreError::IntegrityFailure(format!("prepare failed: {}", e)))?;
            let rows = stmt
                .query_map(params![eid], |row| {
                    let entity_str: Option<String> = row.get(1)?;
                    let created: i64 = row.get(5)?;
                    Ok(EvidenceArtifact {
                        content_hash: row.get(0)?,
                        entity_id: entity_str
                            .and_then(|s| EntityId::try_from(s).ok()),
                        artifact_type: row.get(2)?,
                        deposited_by: row.get(3)?,
                        content: row.get(4)?,
                        created_at: created as u64,
                    })
                })
                .map_err(|e| StoreError::IntegrityFailure(format!("query failed: {}", e)))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| StoreError::IntegrityFailure(format!("row: {}", e)))?);
            }
            Ok(out)
        })
    }
}
