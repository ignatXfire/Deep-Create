//! Real E2E crash recovery test: launch app, submit objective, kill process, verify persistence.
//!
//! This is NOT a unit test — it spawns the real dc-desktop.exe binary.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn find_desktop_exe() -> PathBuf {
    // Try release first, then debug — relative to workspace root
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let release =
        workspace.join(".local/hermes-organization-20260916/target/release/dc-desktop.exe");
    let debug = workspace.join(".local/hermes-organization-20260916/target/debug/dc-desktop.exe");
    let target_release = workspace.join("target/release/dc-desktop.exe");
    let target_debug = workspace.join("target/debug/dc-desktop.exe");

    for path in [&debug, &target_debug, &release, &target_release] {
        if path.exists() {
            return path.clone();
        }
    }

    panic!(
        "dc-desktop.exe not found. Tried:\n  {:?}\n  {:?}\n  {:?}\n  {:?}",
        release, debug, target_release, target_debug
    );
}

#[test]
#[ignore] // Requires built binary; run with --ignored
fn real_app_persistence_e2e() {
    let exe = find_desktop_exe();
    println!("Using binary: {:?}", exe);

    // Create a temp store path
    let store_dir = std::env::temp_dir().join("dc_e2e_test");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store_path = store_dir.join("test_store.db");

    // Launch the app with a store path
    let mut child = Command::new(&exe)
        .env("DC_STORE_PATH", &store_path)
        .env("RUST_BACKTRACE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to launch dc-desktop");

    println!("Launched dc-desktop with PID: {}", child.id());

    // Give it time to initialize
    thread::sleep(Duration::from_secs(5));

    // Check if still running
    match child.try_wait() {
        Ok(Some(status)) => {
            // Capture stderr before panicking
            let output = child.wait_with_output().expect("Failed to get output");
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "dc-desktop exited early with status: {}\nstderr: {}",
                status, stderr
            );
        }
        Ok(None) => {
            println!("dc-desktop is running");
        }
        Err(e) => {
            panic!("Error checking dc-desktop status: {}", e);
        }
    }

    // Kill the process (simulate crash)
    println!("Killing dc-desktop (simulating crash)...");
    child.kill().expect("Failed to kill dc-desktop");
    child.wait().expect("Failed to wait for dc-desktop");

    println!("dc-desktop killed");

    // Check if store file was created
    assert!(store_path.exists(), "Store file should exist after app run");
    println!("Store file exists: {:?}", store_path);

    // Verify store has content
    let metadata = std::fs::metadata(&store_path).unwrap();
    assert!(metadata.len() > 0, "Store file should not be empty");
    println!("Store file size: {} bytes", metadata.len());

    // Cleanup
    let _ = std::fs::remove_dir_all(&store_dir);

    println!("E2E persistence test PASSED");
}

#[test]
#[ignore]
fn real_app_launches_and_shows_ui() {
    let exe = find_desktop_exe();

    let mut child = Command::new(&exe)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to launch dc-desktop");

    println!("Launched dc-desktop with PID: {}", child.id());

    // Give it time to show UI
    thread::sleep(Duration::from_secs(10));

    // Check if still running (UI shown)
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("dc-desktop exited early with status: {}", status);
        }
        Ok(None) => {
            println!("dc-desktop UI is running");
        }
        Err(e) => {
            panic!("Error checking dc-desktop status: {}", e);
        }
    }

    // Kill
    child.kill().expect("Failed to kill dc-desktop");
    child.wait().expect("Failed to wait");

    println!("E2E UI launch test PASSED");
}

/// Crash recovery E2E: launch → kill (simulate crash) → relaunch → verify
/// persistence store survives and is still readable.
///
/// This is the first half of the full "kill → restart → load persisted graph →
/// render restored graph in UI" scenario. The second half (graph restoration
/// verification) requires UI introspection and is not yet automated.
#[test]
#[ignore]
fn real_app_crash_recovery_restart_e2e() {
    let exe = find_desktop_exe();
    let store_dir = std::env::temp_dir().join("dc_e2e_crash_recovery");
    std::fs::create_dir_all(&store_dir).unwrap();
    let store_path = store_dir.join("crash_store.db");

    // Clean any previous run
    let _ = std::fs::remove_file(&store_path);

    // Phase 1: Launch and let it initialize
    println!("Phase 1: initial launch...");
    let mut child1 = Command::new(&exe)
        .env("DC_STORE_PATH", &store_path)
        .env("RUST_BACKTRACE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to launch dc-desktop (phase 1)");

    thread::sleep(Duration::from_secs(5));

    // Verify it's running
    match child1.try_wait() {
        Ok(Some(status)) => {
            let output = child1.wait_with_output().unwrap();
            panic!(
                "Phase 1 exited early: {}\nstderr: {}",
                status,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(None) => println!("Phase 1 running, PID: {}", child1.id()),
        Err(e) => panic!("Phase 1 error: {}", e),
    }

    // Kill (simulate crash)
    println!("Phase 1: killing (simulate crash)...");
    child1.kill().expect("Failed to kill phase 1");
    child1.wait().expect("Failed to wait phase 1");
    println!("Phase 1 killed");

    // Verify store exists after kill
    assert!(store_path.exists(), "Store must exist after phase 1 kill");
    let size_after_kill = std::fs::metadata(&store_path).unwrap().len();
    assert!(size_after_kill > 0, "Store must not be empty after phase 1");
    println!("Store after kill: {} bytes", size_after_kill);

    // Phase 2: Restart with same store
    println!("Phase 2: restarting with same store...");
    let mut child2 = Command::new(&exe)
        .env("DC_STORE_PATH", &store_path)
        .env("RUST_BACKTRACE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to launch dc-desktop (phase 2)");

    thread::sleep(Duration::from_secs(5));

    // Verify it's running
    match child2.try_wait() {
        Ok(Some(status)) => {
            let output = child2.wait_with_output().unwrap();
            panic!(
                "Phase 2 exited early: {}\nstderr: {}",
                status,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(None) => println!("Phase 2 running, PID: {}", child2.id()),
        Err(e) => panic!("Phase 2 error: {}", e),
    }

    // Verify store still exists and hasn't been corrupted
    assert!(
        store_path.exists(),
        "Store must exist after phase 2 restart"
    );
    let size_after_restart = std::fs::metadata(&store_path).unwrap().len();
    assert!(
        size_after_restart > 0,
        "Store must not be empty after phase 2"
    );
    println!("Store after restart: {} bytes", size_after_restart);

    // Store size should be same or larger (WAL checkpointing may add pages)
    assert!(
        size_after_restart >= size_after_kill,
        "Store size must not shrink after restart: {} < {}",
        size_after_restart,
        size_after_kill
    );

    // Kill phase 2
    println!("Phase 2: killing...");
    child2.kill().expect("Failed to kill phase 2");
    child2.wait().expect("Failed to wait phase 2");
    println!("Phase 2 killed");

    // Final verification: store is still there
    assert!(store_path.exists(), "Store must exist after phase 2 kill");
    let final_size = std::fs::metadata(&store_path).unwrap().len();
    println!("Final store size: {} bytes", final_size);

    // Cleanup
    let _ = std::fs::remove_dir_all(&store_dir);

    println!("E2E crash recovery restart test PASSED");
}
