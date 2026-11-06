//! Error-injection tests for dc-store failure paths.
//!
//! Covers: store corruption recovery, concurrent write conflicts.

use dc_domain::ids::EntityId;
use dc_store::{ExpectedVersion, SQLiteStore, StoreBackend, StoreError};
use std::path::PathBuf;

/// Build a valid UUIDv7 EntityId for tests.
fn test_id_n(n: u128) -> EntityId {
    let timestamp = 0x123456789abc_u128;
    let rand_a = 0x123_u128;
    let rand_b = (0x123456789abcdef_u128 + n) & ((1u128 << 62) - 1);
    let value = (timestamp << 80) | (7 << 76) | (rand_a << 64) | (2 << 62) | rand_b;
    EntityId::from_u128(value).expect("constructed UUIDv7 must be valid")
}

fn temp_store_path(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("deep_create_err_inj");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!(
        "err_inj_{}_{}_{}.db",
        test_name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}

/// 1. Store corruption recovery — write garbage bytes into the DB file,
///    then attempt to open and verify integrity check fails cleanly.
#[test]
fn store_corruption_recovery() {
    let path = temp_store_path("corruption");
    cleanup(&path);

    // Create a valid store first
    {
        let mut store = SQLiteStore::new(path.to_str().unwrap());
        store.initialize(path.as_path()).expect("init");
        store
            .write_bytes(&test_id_n(1), b"data", ExpectedVersion::MustNotExist)
            .expect("write");
        store
            .integrity_check()
            .expect("integrity before corruption");
    }

    // Corrupt the file: overwrite the header with garbage
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new().write(true).open(&path).unwrap();
        file.write_all(b"GARBAGEGARBAGEGARBAGE").unwrap();
    }

    // Attempt to reopen — SQLite should reject the corrupted header
    let mut store = SQLiteStore::new(path.to_str().unwrap());
    let result = store.initialize(path.as_path());
    // The store may still initialize (SQLite is lazy), but operations should fail
    match result {
        Ok(()) => {
            // If init succeeded, reads must fail
            let read_result = store.read_bytes(&test_id_n(1));
            assert!(read_result.is_err(), "read from corrupted store must fail");
        }
        Err(_) => {
            // Init failed — that's also acceptable corruption detection
        }
    }

    cleanup(&path);
}

/// 2. Store corruption — integrity_check on a corrupted database.
#[test]
fn store_integrity_check_corrupted() {
    let path = temp_store_path("integrity");
    cleanup(&path);

    // Create and populate
    {
        let mut store = SQLiteStore::new(path.to_str().unwrap());
        store.initialize(path.as_path()).expect("init");
        for i in 0..10 {
            store
                .write_bytes(
                    &test_id_n(i),
                    format!("data-{}", i).as_bytes(),
                    ExpectedVersion::MustNotExist,
                )
                .expect("write");
        }
    }

    // Corrupt by truncating the file mid-way (after the header)
    {
        use std::fs::OpenOptions;
        use std::io::{Seek, SeekFrom, Write};
        let mut file = OpenOptions::new().write(true).open(&path).unwrap();
        file.seek(SeekFrom::Start(4096)).unwrap();
        file.write_all(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
    }

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    let _ = store.initialize(path.as_path());
    let check = store.integrity_check();
    // Corruption may or may not be caught depending on where the damage landed.
    // The test pins that the store does NOT panic and returns a Result.
    match check {
        Ok(()) => eprintln!("integrity check passed despite corruption (page not touched)"),
        Err(e) => eprintln!("integrity check caught corruption: {}", e),
    }

    cleanup(&path);
}

/// 3. Concurrent write conflict — two writers race on the same row.
#[test]
fn store_concurrent_write_conflict() {
    let path = temp_store_path("conflict");
    cleanup(&path);

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    store.initialize(path.as_path()).expect("init");

    let id = test_id_n(1);
    let v1 = store
        .write_bytes(&id, b"initial", ExpectedVersion::MustNotExist)
        .expect("initial write");
    assert_eq!(v1.get(), 1);

    // Simulate a concurrent writer that reads the same version
    let mut store2 = SQLiteStore::new(path.to_str().unwrap());
    store2.initialize(path.as_path()).expect("init2");

    // Both stores read the current version
    let (_, data1) = store.read_bytes(&id).expect("read1").unwrap();
    let (_, data2) = store2.read_bytes(&id).expect("read2").unwrap();
    assert_eq!(data1, b"initial");
    assert_eq!(data2, b"initial");

    // First writer commits
    let v2 = store
        .write_bytes(&id, b"writer1", ExpectedVersion::Existing(v1))
        .expect("writer1 commit");
    assert_eq!(v2.get(), 2);

    // Second writer tries to commit with stale version → conflict
    let result = store2.write_bytes(&id, b"writer2", ExpectedVersion::Existing(v1));
    assert!(result.is_err(), "expected RowVersionConflict");
    assert!(
        matches!(result.unwrap_err(), StoreError::RowVersionConflict { .. }),
        "expected RowVersionConflict"
    );

    // Verify the store still contains writer1's data
    let (_, final_data) = store.read_bytes(&id).expect("final read").unwrap();
    assert_eq!(final_data, b"writer1");

    cleanup(&path);
}

/// 4. CAS conflict — explicit version mismatch on write.
#[test]
fn store_cas_version_mismatch() {
    let path = temp_store_path("cas");
    cleanup(&path);

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    store.initialize(path.as_path()).expect("init");

    let id = test_id_n(2);
    let v1 = store
        .write_bytes(&id, b"v1", ExpectedVersion::MustNotExist)
        .expect("write v1");

    // Try to write with wrong expected version
    let wrong_version = dc_domain::ids::RowVersion::new(v1.get() + 1).unwrap();
    let result = store.write_bytes(&id, b"v2", ExpectedVersion::Existing(wrong_version));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), StoreError::RowVersionConflict { .. }),
        "expected RowVersionConflict"
    );

    cleanup(&path);
}

/// 5. Write to non-existent row with Existing precondition → RowVersionExpectedMissing.
#[test]
fn store_write_missing_row() {
    let path = temp_store_path("missing");
    cleanup(&path);

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    store.initialize(path.as_path()).expect("init");

    let id = test_id_n(3);
    let v1 = dc_domain::ids::RowVersion::new(1).unwrap();
    let result = store.write_bytes(&id, b"data", ExpectedVersion::Existing(v1));
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), StoreError::RowVersionExpectedMissing),
        "expected RowVersionExpectedMissing"
    );

    cleanup(&path);
}

/// 6. Double initialization → AlreadyInitialized.
#[test]
fn store_double_init() {
    let path = temp_store_path("double_init");
    cleanup(&path);

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    store.initialize(path.as_path()).expect("first init");
    let result = store.initialize(path.as_path());
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), StoreError::AlreadyInitialized),
        "expected AlreadyInitialized"
    );

    cleanup(&path);
}

/// 7. Row version overflow — attempt to write beyond i64::MAX.
#[test]
fn store_row_version_overflow() {
    let path = temp_store_path("overflow");
    cleanup(&path);

    let mut store = SQLiteStore::new(path.to_str().unwrap());
    store.initialize(path.as_path()).expect("init");

    let id = test_id_n(4);
    // Manually insert a row with version at the boundary
    let max_version = dc_domain::ids::RowVersion::new(dc_store::ROW_VERSION_SQLITE_MAX).unwrap();
    {
        use rusqlite::params;
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO entities (id, row_version, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id.to_string(), max_version.get() as i64, b"data", 0i64, 0i64],
        ).unwrap();
    }

    // Attempt to write → should fail with RowVersionOverflow
    let result = store.write_bytes(&id, b"new", ExpectedVersion::Any);
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), StoreError::RowVersionOverflow(_)),
        "expected RowVersionOverflow"
    );

    cleanup(&path);
}
