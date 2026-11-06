//! Reproduction tests: PASS means the defect was reproduced, not fixed.
#[cfg(test)]
mod tests {
    use dc_domain::ids::EntityId;
    use dc_store::{EventRecord, SQLiteStore, Stage3Backend, StoreBackend};
    use std::path::Path;

    #[test]
    fn desktop_constant_ids_are_invalid() {
        for raw in [1, 1000, 2000, 3000] {
            assert!(EntityId::from_u128(raw).is_none());
        }
        assert!(std::panic::catch_unwind(|| EntityId::from_u128(1).unwrap()).is_err());
    }

    #[test]
    fn actual_ui_dashed_id_is_rejected() {
        let ui_id = "00000000-0000-7000-8000-000000000001";
        assert!(EntityId::try_from(ui_id.to_owned()).is_err());
        assert!(EntityId::try_from(ui_id.replace('-', "")).is_ok());
    }

    #[test]
    fn duplicate_event_id_silently_replaces_history() {
        let mut store = SQLiteStore::new(":memory:");
        store.initialize(Path::new(":memory:")).unwrap();
        let entity_id = EntityId::from_u128(0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f).unwrap();
        store.write_bytes(&entity_id, b"artificial parent", dc_store::ExpectedVersion::MustNotExist).unwrap();
        let mut event = EventRecord { event_id: "same-id".into(), event_type: "audit".into(), entity_id, payload: b"first".to_vec(), created_at: 1 };
        store.put_event(&event).unwrap();
        event.payload = b"second".to_vec();
        store.put_event(&event).unwrap();
        let events = store.events_for_entity(&entity_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].payload, b"second");
    }

    #[test]
    fn future_schema_version_is_accepted() {
        let dir = std::env::var("AUDIT_TMP").expect("explicit D: artificial data directory");
        let path = Path::new(&dir).join("future-schema-probe.sqlite");
        {
            let mut store = SQLiteStore::new(path.to_str().unwrap());
            store.initialize(&path).unwrap();
        }
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute("UPDATE store_meta SET value='999' WHERE key='schema_version'", []).unwrap();
        }
        let mut store = SQLiteStore::new(path.to_str().unwrap());
        assert!(store.initialize(&path).is_ok(), "current implementation accepts future schema");
    }
}
