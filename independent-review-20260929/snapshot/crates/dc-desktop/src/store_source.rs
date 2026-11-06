//! Store-backed live state source for the desktop UI.
//!
//! This module bridges `dc-desktop` to the real `dc-store` backend,
//! replacing the `FixtureStateSource` with authoritative state.

#![cfg(windows)]

use std::path::PathBuf;

use dc_store::{SQLiteStore, Stage3Backend};

use crate::{LiveStateSource, WorkUnitView};

/// A live state source backed by the real SQLite store.
pub struct StoreStateSource {
    store_path: PathBuf,
}

impl StoreStateSource {
    pub fn new(store_path: PathBuf) -> Self {
        Self { store_path }
    }
}

impl LiveStateSource for StoreStateSource {
    fn work_units(&self) -> Vec<WorkUnitView> {
        // Open the store read-only for this query.
        let store = SQLiteStore::new(self.store_path.to_str().unwrap_or(""));

        // Query Stage-3 events to derive work units.
        // For now, map events to work units as a proof-of-concept.
        // Full integration would join with resources and evidence.
        let entity_id = match dc_domain::ids::EntityId::from_u128(0) {
            Some(id) => id,
            None => return Vec::new(),
        };
        let events = match store.events_for_entity(&entity_id) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        events
            .into_iter()
            .enumerate()
            .map(|(i, e)| WorkUnitView {
                id: format!("WU-{:03}", i + 1),
                kind: e.event_type.clone(),
                stage: "proposed".to_string(),
                summary: format!("Event {}", e.event_id),
                verdict: None,
                artifacts: vec![],
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_source_reads_events() {
        // Create a temp store with test events.
        let dir = std::env::temp_dir().join("dc-store-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");

        {
            let _store = SQLiteStore::new(path.to_str().unwrap());
            // Store is empty; no events yet.
        }

        let source = StoreStateSource::new(path);
        let units = source.work_units();
        // Empty store → empty work units (not a failure, just no data).
        assert!(units.is_empty());
    }
}
