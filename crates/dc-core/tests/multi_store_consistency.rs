//! Integration test: dc-core and dc-watchdog share the same SQLite store
//! for authority.anchor persistence, proving multi-store consistency.

#[cfg(test)]
mod tests {
    use dc_core::persistence::{CorePersistence, PersistedCoreState};
    use dc_domain::ids::{AuthorityEpoch, EntityId, RunGeneration};
    use dc_store::{ExpectedVersion, SQLiteStore, StoreBackend};
    use dc_watchdog::anchor_store::{load_anchor, persist_anchor};
    use dc_watchdog::authority_anchor::AuthorityAnchor;

    fn test_id_n(seed: u64) -> EntityId {
        let mut value = (seed as u128) & 0x3FFFFFFFFFFFFFFF;
        value |= 0x7 << 76;
        value |= 0x2 << 62;
        EntityId::from_u128(value).unwrap()
    }

    #[test]
    fn shared_store_multi_component_consistency() {
        let dir = std::env::temp_dir().join(format!("dc-shared-store-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("shared.db");

        let installation_id = test_id_n(1);

        // Phase 1: dc-core writes epoch 5
        {
            let store = SQLiteStore::new(db_path.to_str().unwrap());
            let mut persistence = CorePersistence::new(store);
            persistence.initialize(&db_path).unwrap();

            // Create parent entity for FK
            persistence
                .persist_state(
                    installation_id,
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
                .record_authority_anchor(installation_id, "core", 5, b"epoch=5;generation=1")
                .unwrap();
        }

        // Phase 2: dc-watchdog writes epoch 10 to the SAME store
        {
            let mut store = SQLiteStore::new(db_path.to_str().unwrap());
            store.initialize(std::path::Path::new(&db_path)).unwrap();
            // Re-create parent entity for FK (Stage-3 tables reference entities)
            store
                .write_bytes(&installation_id, b"installation", ExpectedVersion::Any)
                .unwrap();

            let anchor = AuthorityAnchor {
                installation_id,
                authority_epoch: AuthorityEpoch::new(10).unwrap(),
                run_generation: RunGeneration::new(2).unwrap(),
            };
            persist_anchor(&mut store, &anchor).unwrap();
        }

        // Phase 3: dc-core reads back — should see epoch 10 (monotonic)
        {
            let store = SQLiteStore::new(db_path.to_str().unwrap());
            let mut persistence = CorePersistence::new(store);
            persistence.initialize(&db_path).unwrap();

            let loaded = persistence
                .load_authority_anchor(&installation_id, "watchdog.authority.anchor")
                .unwrap()
                .unwrap();

            // The stored state should contain epoch=10
            let state_str = String::from_utf8_lossy(&loaded.state);
            assert!(
                state_str.contains("epoch=10"),
                "dc-core must see dc-watchdog's epoch 10, got: {}",
                state_str
            );
        }

        // Phase 4: dc-watchdog attempts rollback to epoch 3 — must fail
        {
            let mut store = SQLiteStore::new(db_path.to_str().unwrap());
            store.initialize(std::path::Path::new(&db_path)).unwrap();
            store
                .write_bytes(&installation_id, b"installation", ExpectedVersion::Any)
                .unwrap();

            let stale_anchor = AuthorityAnchor {
                installation_id,
                authority_epoch: AuthorityEpoch::new(3).unwrap(),
                run_generation: RunGeneration::new(99).unwrap(),
            };
            let result = persist_anchor(&mut store, &stale_anchor);
            assert!(
                result.is_err(),
                "rollback to epoch 3 must be refused when epoch 10 exists"
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
