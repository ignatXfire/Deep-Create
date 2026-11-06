//! Integration tests: dc-domain plans persisted through dc-store.
//! 
//! This verifies that explanation and goal_change artifacts can be
//! stored and retrieved without loss of provenance.

use dc_domain::ids::{EntityId, UnitRevision};
use dc_domain::work::{ExplanationArtifact, GoalChangeRecord, GoalChangeRequester, UnitRef};
use dc_store::{Store, StoredEntity};
use std::num::NonZeroU64;
use std::path::Path;

fn test_id(seed: u128) -> EntityId {
    let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
    EntityId::from_u128(base + seed).expect("valid UUIDv7")
}

fn unit_ref(id: EntityId) -> UnitRef {
    UnitRef {
        unit_id: id,
        revision: UnitRevision::MIN,
    }
}

#[test]
fn explanation_artifact_persists_through_store() {
    let mut store = Store::memory();
    store.initialize(Path::new(":memory:")).unwrap();
    
    let unit_id = test_id(1);
    let artifact_id = test_id(100);
    
    let artifact = ExplanationArtifact {
        unit: unit_ref(unit_id),
        content: "The unit is blocked because input data format is ambiguous.".to_string(),
        unit_revision: UnitRevision::MIN,
        explained_at: 42,
    };
    
    let stored = StoredEntity {
        id: artifact_id,
        row_version: NonZeroU64::MIN,
        data: artifact,
        created_at: 1,
        updated_at: 1,
    };
    
    let version = store.write(&stored).unwrap();
    assert_eq!(version, NonZeroU64::MIN);
    
    let retrieved: Option<StoredEntity<ExplanationArtifact>> = store.read(&artifact_id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.data.content, "The unit is blocked because input data format is ambiguous.");
    assert_eq!(retrieved.data.unit.unit_id, unit_id);
}

#[test]
fn goal_change_record_persists_through_store() {
    let mut store = Store::memory();
    store.initialize(Path::new(":memory:")).unwrap();
    
    let unit_id = test_id(2);
    let record_id = test_id(200);
    
    let record = GoalChangeRecord {
        unit: unit_ref(unit_id),
        previous_objective: "Write chapter 1".to_string(),
        new_objective: "Write chapter 1 with focus on character motivation".to_string(),
        reason: "Owner requested deeper character development".to_string(),
        changed_at: 100,
        changed_by: GoalChangeRequester::Owner,
    };
    
    let stored = StoredEntity {
        id: record_id,
        row_version: NonZeroU64::MIN,
        data: record,
        created_at: 1,
        updated_at: 1,
    };
    
    let version = store.write(&stored).unwrap();
    assert_eq!(version, NonZeroU64::MIN);
    
    let retrieved: Option<StoredEntity<GoalChangeRecord>> = store.read(&record_id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.data.previous_objective, "Write chapter 1");
    assert_eq!(retrieved.data.new_objective, "Write chapter 1 with focus on character motivation");
    assert_eq!(retrieved.data.changed_by, GoalChangeRequester::Owner);
}

#[test]
fn multiple_artifacts_for_same_unit() {
    let mut store = Store::memory();
    store.initialize(Path::new(":memory:")).unwrap();
    
    let unit_id = test_id(3);
    
    // Store explanation
    let explanation_id = test_id(301);
    let explanation = ExplanationArtifact {
        unit: unit_ref(unit_id),
        content: "Clarification needed on scope".to_string(),
        unit_revision: UnitRevision::MIN,
        explained_at: 1,
    };
    store.write(&StoredEntity {
        id: explanation_id,
        row_version: NonZeroU64::MIN,
        data: explanation,
        created_at: 1,
        updated_at: 1,
    }).unwrap();
    
    // Store goal change
    let goal_change_id = test_id(302);
    let goal_change = GoalChangeRecord {
        unit: unit_ref(unit_id),
        previous_objective: "Original".to_string(),
        new_objective: "Updated".to_string(),
        reason: "Scope narrowed".to_string(),
        changed_at: 2,
        changed_by: GoalChangeRequester::System,
    };
    store.write(&StoredEntity {
        id: goal_change_id,
        row_version: NonZeroU64::MIN,
        data: goal_change,
        created_at: 2,
        updated_at: 2,
    }).unwrap();
    
    // Both retrievable independently
    let exp: Option<StoredEntity<ExplanationArtifact>> = store.read(&explanation_id).unwrap();
    let gc: Option<StoredEntity<GoalChangeRecord>> = store.read(&goal_change_id).unwrap();
    
    assert!(exp.is_some());
    assert!(gc.is_some());
    assert_eq!(exp.unwrap().data.unit.unit_id, unit_id);
    assert_eq!(gc.unwrap().data.unit.unit_id, unit_id);
}

#[test]
fn store_integrity_after_multiple_writes() {
    let mut store = Store::memory();
    store.initialize(Path::new(":memory:")).unwrap();
    
    // Write 10 artifacts
    for i in 0..10u128 {
        let id = test_id(400 + i);
        let unit_id = test_id(500 + i);
        let artifact = ExplanationArtifact {
            unit: unit_ref(unit_id),
            content: format!("Explanation {}", i),
            unit_revision: UnitRevision::MIN,
            explained_at: i as u64,
        };
        store.write(&StoredEntity {
            id,
            row_version: NonZeroU64::MIN,
            data: artifact,
            created_at: i as u64,
            updated_at: i as u64,
        }).unwrap();
    }
    
    // Verify all exist
    for i in 0..10u128 {
        let id = test_id(400 + i);
        assert!(store.exists(&id).unwrap());
    }
    
    // Integrity check passes
    store.integrity_check().unwrap();
}
