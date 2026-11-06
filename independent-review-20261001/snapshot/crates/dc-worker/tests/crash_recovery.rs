//! Real crash recovery test: kills the worker process mid-job with an OS-level
//! hard kill (SIGKILL equivalent), restarts it, and verifies state recovered
//! from the durable WAL-backed journal in dc-store.
//!
//! Design (Cano / Duroxide / durust write-ahead checkpoint pattern):
//!   1. Child process writes a job entity + JobStarted recovery checkpoint
//!      (write-ahead) BEFORE doing any "work".
//!   2. Child then simulates work (progress checkpoints).
//!   3. Parent observes JobStarted in the store (proves WAL commit landed),
//!      then hard-kills the child (TerminateProcess — the Windows SIGKILL
//!      analog; no atexit, no Drop, no flush).
//!   4. Parent re-spawns the child in recover mode; the child reads
//!      latest_recovery_checkpoint, verifies it is JobStarted (not
//!      JobCompleted), performs idempotent replay (writes JobRecovered),
//!      prints a JSON verdict to stdout.
//!   5. Parent asserts verdict.success == true.
//!
//! Idempotency: replay is safe because the recovery handler keys off the
//! latest checkpoint type — re-running it on an already-recovered job is a
//! no-op (the child checks for JobRecovered before writing it again).
//!
//! Re-exec pattern: the child is THIS test binary, re-spawned with the env
//! var DC_CRASH_CHILD set, so no separate dc-worker executable needs to be
//! built. That keeps the test self-contained while still proving the real
//! thing: a separate OS process, a real file, a real kill, a real reopen.

use std::env;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use dc_domain::ids::EntityId;
use dc_store::{ExpectedVersion, RecoveryCheckpoint, SQLiteStore, Stage3Backend, StoreBackend};

const ENV_CHILD: &str = "DC_CRASH_CHILD";
const ENV_DB_PATH: &str = "DC_CRASH_DB";
const ENV_JOB_ID: &str = "DC_CRASH_JOB";
const ENV_MODE: &str = "DC_CRASH_MODE";

/// Deterministic UUIDv7-shaped entity id for the job row.
fn job_entity_id(seed: u64) -> EntityId {
    let hi: u128 = 0x018f_1234_5678_7abc;
    let lo: u128 = 0x8000_0000_0000_0000 | (seed as u128 & 0x3fff_ffff_ffff_ffff);
    let value: u128 = (hi << 64) | lo;
    EntityId::from_u128(value).expect("must satisfy UUIDv7 layout")
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn open_store(db_path: &str) -> SQLiteStore {
    let mut store = SQLiteStore::new(db_path);
    store
        .initialize(std::path::Path::new(db_path))
        .expect("initialize store");
    store
}

fn child_write_ahead(store: &mut SQLiteStore, job: &EntityId) {
    let payload = br#"{"kind":"crash_recovery_job","state":"pending"}"#;
    store.begin_transaction().expect("child: begin txn");
    store
        .write_bytes(job, payload, ExpectedVersion::MustNotExist)
        .expect("child: insert job entity");
    store.commit().expect("child: commit job entity");

    store
        .put_recovery_checkpoint(&RecoveryCheckpoint {
            checkpoint_id: "ckpt-started-1".into(),
            entity_id: *job,
            checkpoint_type: "JobStarted".into(),
            state: br#"{"phase":"started"}"#.to_vec(),
            created_at: now_ms(),
        })
        .expect("child: write JobStarted checkpoint");

    println!("CHILD: write-ahead complete (job + JobStarted)");
}

fn child_do_work(store: &mut SQLiteStore, job: &EntityId) {
    for i in 0..10u32 {
        thread::sleep(Duration::from_millis(500));
        store
            .put_recovery_checkpoint(&RecoveryCheckpoint {
                checkpoint_id: format!("ckpt-progress-{}", i),
                entity_id: *job,
                checkpoint_type: "JobProgress".into(),
                state: format!(r#"{{"phase":"progress","i":{}}}"#, i).into_bytes(),
                created_at: now_ms(),
            })
            .expect("child: write JobProgress checkpoint");
        println!("CHILD: progress i={}", i);
    }
    store
        .put_recovery_checkpoint(&RecoveryCheckpoint {
            checkpoint_id: "ckpt-completed-1".into(),
            entity_id: *job,
            checkpoint_type: "JobCompleted".into(),
            state: br#"{"phase":"completed"}"#.to_vec(),
            created_at: now_ms(),
        })
        .expect("child: write JobCompleted checkpoint");
    println!("CHILD: completed (this should NOT be reached in the kill scenario)");
}

fn child_recover(store: &mut SQLiteStore, job: &EntityId) -> serde_json::Value {
    let latest = store
        .latest_recovery_checkpoint(job)
        .expect("recover: read latest checkpoint");

    match latest {
        None => serde_json::json!({
            "success": false,
            "reason": "no checkpoint found after crash"
        }),
        Some(ck) => match ck.checkpoint_type.as_str() {
            "JobCompleted" => serde_json::json!({
                "success": false,
                "reason": "found JobCompleted — kill did not happen mid-job",
                "found": ck.checkpoint_type,
            }),
            "JobRecovered" => serde_json::json!({
                "success": true,
                "idempotent_noop": true,
                "found": ck.checkpoint_type,
            }),
            "JobStarted" | "JobProgress" => {
                store
                    .put_recovery_checkpoint(&RecoveryCheckpoint {
                        checkpoint_id: "ckpt-recovered-1".into(),
                        entity_id: *job,
                        checkpoint_type: "JobRecovered".into(),
                        state: format!(
                            r#"{{"phase":"recovered","replayed_from":"{}"}}"#,
                            ck.checkpoint_type
                        )
                        .into_bytes(),
                        created_at: now_ms(),
                    })
                    .expect("recover: write JobRecovered");
                serde_json::json!({
                    "success": true,
                    "idempotent_noop": false,
                    "replayed_from": ck.checkpoint_type,
                    "found": ck.checkpoint_type,
                })
            }
            other => serde_json::json!({
                "success": false,
                "reason": "unknown checkpoint type",
                "found": other,
            }),
        },
    }
}

fn run_child() -> ! {
    let db_path = env::var(ENV_DB_PATH).expect("child: DC_CRASH_DB");
    let job_str = env::var(ENV_JOB_ID).expect("child: DC_CRASH_JOB");
    let mode = env::var(ENV_MODE).unwrap_or_else(|_| "work".to_string());
    let job = EntityId::try_from(job_str).expect("child: parse job id");

    let mut store = open_store(&db_path);

    match mode.as_str() {
        "work" => {
            child_write_ahead(&mut store, &job);
            child_do_work(&mut store, &job);
            eprintln!("CHILD: ERROR — survived work loop");
            std::process::exit(2);
        }
        "recover" => {
            let verdict = child_recover(&mut store, &job);
            println!("VERDICT:{}", serde_json::to_string(&verdict).unwrap());
            let code = if verdict["success"].as_bool().unwrap_or(false) {
                0
            } else {
                3
            };
            std::process::exit(code);
        }
        other => {
            eprintln!("CHILD: unknown mode {}", other);
            std::process::exit(4);
        }
    }
}

fn spawn_child(db_path: &str, job: &EntityId, mode: &str) -> std::process::Child {
    let exe = env::current_exe().expect("current_exe");
    let mut cmd = Command::new(exe);
    cmd.env(ENV_CHILD, "1")
        .env(ENV_DB_PATH, db_path)
        .env(ENV_JOB_ID, job.to_string())
        .env(ENV_MODE, mode)
        .arg("--exact")
        .arg("crash_recovery_child_entry")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn().expect("spawn child")
}

/// Hard OS-level kill. On Windows, std::process::Child::kill() maps to
/// TerminateProcess — the SIGKILL analog: no handlers, no Drop, no flush.
fn hard_kill(child: &mut std::process::Child) {
    child.kill().expect("hard kill");
    let _ = child.wait();
}

#[test]
fn crash_recovery_child_entry() {
    // This test function exists ONLY to be re-exec'd as the child process.
    if env::var(ENV_CHILD).is_ok() {
        run_child();
    }
}

#[test]
fn crash_recovery_kill_and_restart() {
    let tmp_root = env::temp_dir().join(format!("dc-crash-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_root).expect("create tmp dir");
    let db_path = tmp_root.join("recovery.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));

    let job = job_entity_id(0xdeadc0de);

    // Pre-initialize the schema so the child does not race schema creation.
    {
        let mut s = open_store(&db_path_str);
        s.begin_transaction().expect("parent: init txn");
        let seed_id = job_entity_id(0x5eed);
        s.write_bytes(&seed_id, b"seed", ExpectedVersion::MustNotExist)
            .expect("parent: seed write");
        s.commit().expect("parent: seed commit");
        let _ = s.read_bytes(&seed_id).expect("parent: seed read");
    }

    // Phase 1: spawn worker, wait for write-ahead to land.
    let mut child = spawn_child(&db_path_str, &job, "work");
    let child_pid = child.id();
    println!("PARENT: spawned worker pid={}", child_pid);

    let read_store = open_store(&db_path_str);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut seen_started = false;
    while Instant::now() < deadline {
        if let Ok(Some(ck)) = read_store.latest_recovery_checkpoint(&job) {
            if ck.checkpoint_type == "JobStarted" {
                seen_started = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        seen_started,
        "parent never observed JobStarted checkpoint — write-ahead failed"
    );
    println!("PARENT: observed JobStarted in WAL");

    thread::sleep(Duration::from_millis(600));

    // Phase 2: hard kill.
    hard_kill(&mut child);
    println!("PARENT: killed worker pid={}", child_pid);

    // Phase 3: restart in recover mode.
    let recover = spawn_child(&db_path_str, &job, "recover");
    let output = recover
        .wait_with_output()
        .expect("recover: wait_with_output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("RECOVER stdout:\n{}", stdout);
    println!("RECOVER stderr:\n{}", stderr);

    assert!(
        output.status.success(),
        "recover child exited non-zero: {:?}",
        output.status
    );

    let verdict_pos = stdout
        .find("VERDICT:")
        .expect("no VERDICT marker in recover stdout");
    let verdict_str = stdout[verdict_pos + "VERDICT:".len()..]
        .lines()
        .next()
        .unwrap_or("")
        .trim();
    let verdict: serde_json::Value = serde_json::from_str(verdict_str).expect("parse verdict");
    assert_eq!(
        verdict["success"].as_bool(),
        Some(true),
        "recovery verdict not successful: {}",
        verdict
    );
    let replayed_from = verdict["replayed_from"].as_str().unwrap_or("");
    assert!(
        replayed_from == "JobStarted" || replayed_from == "JobProgress",
        "expected replay from JobStarted or JobProgress, got: {}",
        verdict
    );

    // Phase 4: verify recovered state is durable.
    let final_ck = read_store
        .latest_recovery_checkpoint(&job)
        .expect("parent: final read")
        .expect("parent: checkpoint must exist");
    assert_eq!(
        final_ck.checkpoint_type, "JobRecovered",
        "expected JobRecovered, got {}",
        final_ck.checkpoint_type
    );

    let _ = std::fs::remove_dir_all(&tmp_root);
    println!("PARENT: PASS — write-ahead + kill + restart + replay verified");
}
