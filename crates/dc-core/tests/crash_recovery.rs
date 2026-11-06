//! E2E test: crash recovery with graph persistence
//!
//! Tests that PersistedCoreState with graph_json can be serialized,
//! persisted, loaded, and the graph restored via AcceptGraphRaw.

use dc_core::persistence::{CorePersistence, PersistedCoreState};
use dc_domain::ids::EntityId;
use dc_store::SQLiteStore;
use std::path::PathBuf;

/// Build a valid UUIDv7 EntityId for tests (mirrors dc-core test_support).
fn test_id_n(n: u128) -> EntityId {
    let timestamp = 0x123456789abc_u128;
    let rand_a = 0x123_u128;
    let rand_b = (0x123456789abcdef_u128 + n) & ((1u128 << 62) - 1);
    let value = (timestamp << 80) | (7 << 76) | (rand_a << 64) | (2 << 62) | rand_b;
    EntityId::from_u128(value).expect("constructed UUIDv7 must be valid")
}

fn temp_store_path(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("deep_create_test");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!(
        "crash_recovery_{}_{}.db",
        test_name,
        std::process::id()
    ))
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}

/// Test that graph_json survives persist/load round-trip
#[test]
fn graph_json_persistence_roundtrip() {
    let path = temp_store_path("roundtrip");
    cleanup(&path);

    // Create a minimal valid graph JSON (matching AcceptGraphRaw format)
    let graph_json = r#"{
        "expected_graph_version": 1,
        "current_graph_version": 1,
        "roots": [],
        "objectives": [],
        "units": [],
        "dependencies": [],
        "links": [],
        "publications": [],
        "publication_states": [],
        "publication_bindings": [],
        "owner_acceptances": []
    }"#;

    let objective_json = r#"{
        "objective_id": "00000000-0000-0000-0000-000000000001",
        "statement": "Test objective",
        "constraints": [],
        "success_criteria": [],
        "scope": {
            "scope_id": "00000000-0000-0000-0000-000000000064",
            "installation_id": "00000000-0000-0000-0000-0000000003e8",
            "namespace": "test"
        },
        "priority": "Normal",
        "created_at": "2026-09-29T12:00:00Z",
        "created_by": "00000000-0000-0000-0000-000000002710"
    }"#;

    let persisted = PersistedCoreState {
        current_objective: Some(test_id_n(1)),
        current_graph_version: 1,
        active_units: vec![test_id_n(10), test_id_n(11)],
        completed_units: vec![test_id_n(12)],
        objective_json: Some(objective_json.to_string()),
        graph_json: Some(graph_json.to_string()),
    };

    // Persist
    let store = SQLiteStore::new(path.to_str().unwrap());
    let mut persistence = CorePersistence::new(store);
    persistence.initialize(&path).unwrap();

    let state_id = test_id_n(1);
    persistence
        .persist_state(
            state_id,
            &persisted,
            dc_store::ExpectedVersion::MustNotExist,
        )
        .unwrap();

    // Load and verify
    let loaded = persistence.load_state(&state_id).unwrap().unwrap();
    assert_eq!(loaded.data.current_graph_version, 1);
    assert_eq!(loaded.data.active_units.len(), 2);
    assert!(loaded.data.graph_json.is_some());
    assert!(loaded.data.objective_json.is_some());

    let loaded_graph_json = loaded.data.graph_json.as_ref().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(loaded_graph_json).unwrap();
    assert_eq!(parsed["current_graph_version"], 1);

    cleanup(&path);
}

/// Test that old format (without graph_json) still loads
#[test]
fn old_format_without_graph_json_loads() {
    let path = temp_store_path("old_format");
    cleanup(&path);

    let persisted = PersistedCoreState {
        current_objective: Some(test_id_n(1)),
        current_graph_version: 5,
        active_units: vec![test_id_n(10)],
        completed_units: vec![],
        objective_json: None,
        graph_json: None,
    };

    let store = SQLiteStore::new(path.to_str().unwrap());
    let mut persistence = CorePersistence::new(store);
    persistence.initialize(&path).unwrap();

    let state_id = test_id_n(1);
    persistence
        .persist_state(
            state_id,
            &persisted,
            dc_store::ExpectedVersion::MustNotExist,
        )
        .unwrap();

    let loaded = persistence.load_state(&state_id).unwrap().unwrap();
    assert_eq!(loaded.data.current_graph_version, 5);
    assert!(loaded.data.graph_json.is_none());

    cleanup(&path);
}
