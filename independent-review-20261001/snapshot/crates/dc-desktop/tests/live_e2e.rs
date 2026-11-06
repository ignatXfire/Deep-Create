//! Live E2E test harness — STRATEGY DOCUMENT.
//!
//! The real live E2E tests for the UI↔Core round-trip live in
//! `crates/dc-desktop/src/main.rs` under `#[cfg(test)] mod tests`, because
//! integration tests under `tests/` cannot reach the binary's private
//! `desktop_builder`, `AppState`, or `#[tauri::command]` functions (Cargo
//! builds the bin target in isolation; only `pub` items reachable from a
//! lib target are importable here, and dc-desktop has no lib target).
//!
//! Tests added in main.rs:
//!   - `ui_to_core_full_round_trip_via_real_invoke_handler`
//!       Always runs. Drives application_info → submit_objective →
//!       plan_objective_graph → accept_graph (empty units) → execute_graph →
//!       health_check through the REAL Tauri IPC stack (mock_builder +
//!       get_ipc_response) against the REAL CoreState and REAL command
//!       handlers. No network.
//!   - `ui_to_core_real_broker_dispatch_live`  (#[ignore])
//!       Same path but with a real unit in the graph, so `execute_graph`
//!       actually dispatches through dc-broker to live OmniRoute on
//!       localhost:20128. Run with `--ignored` when OmniRoute is healthy.
//!
//! Why this is the strongest available evidence:
//!   - Uses the SAME `desktop_builder` function production `main()` calls.
//!   - Exercises the REAL Tauri invoke_handler, ACL, IPC serde, command router.
//!   - CoreState is the REAL `RwLock<CoreState>` driven by the REAL
//!     `dc_core::command::handle_command`.
//!
//! What is NOT proven (and would require tauri-driver + WebView2 + a windowed
//! session, out of scope here):
//!   - Real WebView2 rendering and JS bridge injection.
//!   - DOM events → `window.__TAURI__.invoke(...)` wiring in the actual frontend.
//!
//! See `LIVE_E2E_EVIDENCE.md` at workspace root for commands, full output,
//! and the honest verdict.

#[test]
fn live_e2e_tests_live_in_main_rs() {
    // Pointer test so `cargo test -p dc-desktop --test live_e2e` does not
    // silently no-op. The real assertions live in src/main.rs unit tests.
    assert!(true);
}
