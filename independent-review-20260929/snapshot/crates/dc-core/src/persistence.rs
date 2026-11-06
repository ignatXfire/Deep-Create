//! C-03 persistence: dc-core integration with dc-store.
//!
//! This module provides the persistence layer for dc-core commands, states,
//! and results. It uses dc-store's StoreBackend for CAS semantics.

use std::path::Path;

use dc_domain::ids::EntityId;
use dc_domain::work::{WorkUnitSpec, ResultPacket, VerificationRecord};
use dc_store::{
    AuthorityAnchor, EventRecord, EvidenceRecord, ExpectedVersion, RecoveryCheckpoint, Stage3Backend, StoreBackend, StoreError, StoredEntity,
};
use serde::Serialize;

use crate::error::CoreError;

/// Persistence errors specific to dc-core
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("entity not found: {0}")]
    NotFound(EntityId),
    
    #[error("row version conflict")]
    RowVersionConflict,
}

impl From<PersistenceError> for CoreError {
    fn from(err: PersistenceError) -> Self {
        CoreError::Persistence(err.to_string())
    }
}

/// Core state that can be persisted
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PersistedCoreState {
    pub current_objective: Option<EntityId>,
    pub current_graph_version: u64,
    pub active_units: Vec<EntityId>,
    pub completed_units: Vec<EntityId>,
    /// Serialized ObjectiveContract JSON for crash recovery
    pub objective_json: Option<String>,
    /// Serialized GraphCandidate JSON for crash recovery (C-08: only core deserializes)
    pub graph_json: Option<String>,
}

/// Core persistence layer
pub struct CorePersistence<S: StoreBackend> {
    store: S,
}

impl<S: StoreBackend> CorePersistence<S> {
    /// Create a new persistence layer with the given store
    pub fn new(store: S) -> Self {
        Self { store }
    }
    
    /// Initialize the persistence layer
    pub fn initialize(&mut self, path: &Path) -> Result<(), PersistenceError> {
        self.store.initialize(path)?;
        Ok(())
    }
    
    /// Check if persistence is initialized
    pub fn is_initialized(&self) -> bool {
        self.store.is_initialized()
    }
    
    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<(), PersistenceError> {
        self.store.begin_transaction()?;
        Ok(())
    }
    
    /// Commit the current transaction
    pub fn commit(&mut self) -> Result<(), PersistenceError> {
        self.store.commit()?;
        Ok(())
    }
    
    /// Rollback the current transaction
    pub fn rollback(&mut self) -> Result<(), PersistenceError> {
        self.store.rollback()?;
        Ok(())
    }
    
    /// Persist core state
    pub fn persist_state(
        &mut self,
        state_id: EntityId,
        state: &PersistedCoreState,
        expected: ExpectedVersion,
    ) -> Result<u64, PersistenceError> {
        let data = serde_json::to_vec(state)?;
        let row_version = self.store.write_bytes(&state_id, &data, expected)?;
        Ok(row_version.get())
    }
    
    /// Load core state
    pub fn load_state(
        &self,
        state_id: &EntityId,
    ) -> Result<Option<StoredEntity<PersistedCoreState>>, PersistenceError> {
        match self.store.read_bytes(state_id)? {
            Some((row_version, data)) => {
                let state: PersistedCoreState = serde_json::from_slice(&data)?;
                Ok(Some(StoredEntity {
                    id: *state_id,
                    row_version,
                    data: state,
                    created_at: 0, // TODO: Add timestamps
                    updated_at: 0,
                }))
            }
            None => Ok(None),
        }
    }
    
    /// Persist a work unit
    pub fn persist_unit(
        &mut self,
        unit: &WorkUnitSpec,
        expected: ExpectedVersion,
    ) -> Result<u64, PersistenceError> {
        let data = serde_json::to_vec(unit)?;
        let row_version = self.store.write_bytes(&unit.unit.unit_id, &data, expected)?;
        Ok(row_version.get())
    }
    
    /// Load a work unit
    pub fn load_unit(
        &self,
        unit_id: &EntityId,
    ) -> Result<Option<StoredEntity<WorkUnitSpec>>, PersistenceError> {
        match self.store.read_bytes(unit_id)? {
            Some((row_version, data)) => {
                let unit: WorkUnitSpec = serde_json::from_slice(&data)?;
                Ok(Some(StoredEntity {
                    id: *unit_id,
                    row_version,
                    data: unit,
                    created_at: 0,
                    updated_at: 0,
                }))
            }
            None => Ok(None),
        }
    }
    
    /// Persist a result packet
    pub fn persist_result(
        &mut self,
        result: &ResultPacket,
        expected: ExpectedVersion,
    ) -> Result<u64, PersistenceError> {
        let data = serde_json::to_vec(result)?;
        let row_version = self.store.write_bytes(&result.result_id, &data, expected)?;
        Ok(row_version.get())
    }
    
    /// Load a result packet
    pub fn load_result(
        &self,
        result_id: &EntityId,
    ) -> Result<Option<StoredEntity<ResultPacket>>, PersistenceError> {
        match self.store.read_bytes(result_id)? {
            Some((row_version, data)) => {
                let result: ResultPacket = serde_json::from_slice(&data)?;
                Ok(Some(StoredEntity {
                    id: *result_id,
                    row_version,
                    data: result,
                    created_at: 0,
                    updated_at: 0,
                }))
            }
            None => Ok(None),
        }
    }
    
    /// Persist a verification record
    /// 
    /// Note: VerificationRecord does not implement Deserialize (by design).
    /// We serialize it for storage, but cannot deserialize it back.
    /// For persistence, we store the verification_id and result_id only.
    pub fn persist_verification(
        &mut self,
        verification: &VerificationRecord,
        expected: ExpectedVersion,
    ) -> Result<u64, PersistenceError> {
        // Store only the IDs, not the full record (which can't be deserialized)
        #[derive(Serialize)]
        struct VerificationRef {
            verification_id: EntityId,
            result_id: EntityId,
        }
        
        let ref_data = VerificationRef {
            verification_id: verification.verification_id,
            result_id: verification.result_id,
        };
        
        let data = serde_json::to_vec(&ref_data)?;
        let row_version = self.store.write_bytes(&verification.verification_id, &data, expected)?;
        Ok(row_version.get())
    }
    
    /// Run integrity check
    pub fn integrity_check(&self) -> Result<(), PersistenceError> {
        self.store.integrity_check()?;
        Ok(())
    }
}

impl<S: Stage3Backend> CorePersistence<S> {
    // ─────────────────────────────────────────────────────────────────────────
    // Stage 3 audit wiring: every significant dc-core action writes a typed
    // record alongside the authoritative entity. These records live in the
    // Stage-3 tables (events / evidence / recovery / authority.anchor).
    // ─────────────────────────────────────────────────────────────────────────

    /// Record an audit event for an entity (e.g. "command_processed",
    /// "unit_dispatched", "verification_completed").
    pub fn record_event(
        &mut self,
        entity_id: EntityId,
        event_type: &str,
        payload: &[u8],
    ) -> Result<(), PersistenceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.store.put_event(&EventRecord {
            event_id: format!("ev-{}-{}", entity_id, now),
            event_type: event_type.to_string(),
            entity_id,
            payload: payload.to_vec(),
            created_at: now,
        })?;
        Ok(())
    }

    /// Persist a verification evidence record for an entity.
    pub fn record_evidence(
        &mut self,
        entity_id: EntityId,
        verifier_id: &str,
        evidence_type: &str,
        payload: &[u8],
    ) -> Result<(), PersistenceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.store.put_evidence(&EvidenceRecord {
            evidence_id: format!("evd-{}-{}", entity_id, now),
            entity_id,
            evidence_type: evidence_type.to_string(),
            verifier_id: verifier_id.to_string(),
            payload: payload.to_vec(),
            verified_at: now,
            expires_at: None,
        })?;
        Ok(())
    }

    /// Persist a crash-recovery checkpoint for an entity.
    pub fn record_recovery_checkpoint(
        &mut self,
        entity_id: EntityId,
        checkpoint_type: &str,
        state: &[u8],
    ) -> Result<(), PersistenceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.store.put_recovery_checkpoint(&RecoveryCheckpoint {
            checkpoint_id: format!("cp-{}-{}", entity_id, now),
            entity_id,
            checkpoint_type: checkpoint_type.to_string(),
            state: state.to_vec(),
            created_at: now,
        })?;
        Ok(())
    }

    /// Persist an authority anchor for crash recovery (anti-rollback).
    pub fn record_authority_anchor(
        &mut self,
        entity_id: EntityId,
        fence_scope: &str,
        row_version: u64,
        state: &[u8],
    ) -> Result<(), PersistenceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.store.put_authority_anchor(&AuthorityAnchor {
            anchor_id: format!("aa-{}-{}", entity_id, fence_scope),
            entity_id,
            fence_scope: fence_scope.to_string(),
            row_version,
            state: state.to_vec(),
            created_at: now,
        })?;
        Ok(())
    }

    /// Load the latest authority anchor for crash recovery.
    pub fn load_authority_anchor(
        &self,
        entity_id: &EntityId,
        fence_scope: &str,
    ) -> Result<Option<AuthorityAnchor>, PersistenceError> {
        self.store
            .latest_authority_anchor(entity_id, fence_scope)
            .map_err(PersistenceError::Store)
    }

    /// Load the latest recovery checkpoint for an entity.
    pub fn load_latest_recovery_checkpoint(
        &self,
        entity_id: &EntityId,
    ) -> Result<Option<RecoveryCheckpoint>, PersistenceError> {
        self.store
            .latest_recovery_checkpoint(entity_id)
            .map_err(PersistenceError::Store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_store::MemoryStore;
    use crate::test_support::test_id_n;
    
    #[test]
    fn persistence_state_round_trip() {
        let store = MemoryStore::new();
        let mut persistence = CorePersistence::new(store);
        persistence.initialize(Path::new(":memory:")).unwrap();
        
        let state_id = test_id_n(100);
        let state = PersistedCoreState {
            current_objective: Some(test_id_n(101)),
            current_graph_version: 1,
            active_units: vec![test_id_n(102)],
            completed_units: vec![test_id_n(103)],
            objective_json: None,
            graph_json: None,
        };
        
        // Persist
        let version = persistence.persist_state(
            state_id,
            &state,
            ExpectedVersion::MustNotExist,
        ).unwrap();
        assert_eq!(version, 1);
        
        // Load
        let loaded = persistence.load_state(&state_id).unwrap().unwrap();
        assert_eq!(loaded.data.current_objective, state.current_objective);
        assert_eq!(loaded.data.current_graph_version, state.current_graph_version);
        assert_eq!(loaded.data.active_units, state.active_units);
        assert_eq!(loaded.data.completed_units, state.completed_units);
    }
    
    #[test]
    fn persistence_unit_round_trip() {
        let store = MemoryStore::new();
        let mut persistence = CorePersistence::new(store);
        persistence.initialize(Path::new(":memory:")).unwrap();
        
        let unit = crate::test_support::test_unit_spec(test_id_n(200));
        
        // Persist
        let version = persistence.persist_unit(
            &unit,
            ExpectedVersion::MustNotExist,
        ).unwrap();
        assert_eq!(version, 1);
        
        // Load
        let loaded = persistence.load_unit(&unit.unit.unit_id).unwrap().unwrap();
        assert_eq!(loaded.data.unit.unit_id, unit.unit.unit_id);
    }

    /// Stage-3 authority anchor round-trip through the real SQLite store.
    /// This proves the crash-recovery path: persist anchor → drop the
    /// persistence handle → re-open → load the anchor back.
    #[test]
    fn persistence_authority_anchor_round_trip() {
        use dc_store::SQLiteStore;

        let dir = std::env::temp_dir().join(format!(
            "dc-core-anchor-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("anchor.db");

        let entity = test_id_n(300);
        let scope = "core";
        let state = b"epoch=7;generation=3";

        // Write phase
        {
            let store = SQLiteStore::new(db_path.to_str().unwrap());
            let mut persistence = CorePersistence::new(store);
            persistence.initialize(&db_path).unwrap();

            // The Stage-3 tables have a FK to entities(id) — persist a dummy
            // entity first so the anchor can reference it.
            persistence
                .persist_state(
                    entity,
                    &PersistedCoreState {
                        current_objective: None,
                        current_graph_version: 1,
                        active_units: vec![],
                        completed_units: vec![],
                        objective_json: None,
                        graph_json: None,
                    },
                    ExpectedVersion::MustNotExist,
                )
                .unwrap();

            persistence
                .record_authority_anchor(entity, scope, 7, state)
                .unwrap();

            // Recovery checkpoint also round-trips through the same store.
            persistence
                .record_recovery_checkpoint(entity, "pre-crash", b"snapshot")
                .unwrap();
        }

        // Read phase — new handle, same file (simulates post-crash restart)
        {
            let store = SQLiteStore::new(db_path.to_str().unwrap());
            let mut persistence = CorePersistence::new(store);
            persistence.initialize(&db_path).unwrap();

            let anchor = persistence
                .load_authority_anchor(&entity, scope)
                .unwrap()
                .expect("anchor must survive restart");
            assert_eq!(anchor.row_version, 7);
            assert_eq!(anchor.state, state);

            // Wrong scope returns nothing.
            assert!(persistence
                .load_authority_anchor(&entity, "other-scope")
                .unwrap()
                .is_none());

            // Recovery checkpoint also survives.
            let cp = persistence
                .load_latest_recovery_checkpoint(&entity)
                .unwrap()
                .expect("checkpoint must survive restart");
            assert_eq!(cp.checkpoint_type, "pre-crash");
            assert_eq!(cp.state, b"snapshot");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
