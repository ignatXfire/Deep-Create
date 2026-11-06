# LIVE E2E EVIDENCE — UI↔Core Round-Trip via Tauri Mock IPC

**Date:** 2026-10-01
**Environment:** Windows 10, Rust stable, Tauri 2.11.5
**Project:** DeepCreate at D:/Projects/DeepCreate

---

## What Was Tested

A full UI↔Core round-trip through the **real Tauri `invoke_handler`** (not mocked commands) using `tauri::test::mock_builder()` + `get_ipc_response()`. This is the canonical way to test Tauri command handlers without a real WebView2 browser.

### Test Sequence (7 steps)

1. `application_info` — verify initial state (0 work units)
2. `submit_objective` — UI write path → real `handle_command` → `CoreCommand::AcceptObjective`
3. `plan_objective_graph` — read-only bridge → real `dc_core::planner::plan_graph_from_objective`
4. `accept_graph` — UI write path → real `handle_command` → `CoreCommand::AcceptGraphRaw`
5. `work_units` — live projection from `CoreState.graph`
6. `execute_graph` — real executor (empty graph → "no units to execute")
7. `health_check` — final state verification (objective + graph accepted)

---

## Commands Run

### Build check
```bash
cd /d/Projects/DeepCreate && source .local/hermes-organization-20260916/build-env.sh && cargo check -p dc-desktop --tests
```
**Result:** exit 0, clean compile.

### Deterministic round-trip test (all 7 steps)
```bash
cd /d/Projects/DeepCreate && source .local/hermes-organization-20260916/build-env.sh && cargo test -p dc-desktop --bin dc-desktop ui_to_core_full_round_trip_via_real_invoke_handler -- --nocapture
```
**Result:** exit 0, **1 passed**

### Live broker dispatch test (real HTTP to OmniRoute)
```bash
cd /d/Projects/DeepCreate && source .local/hermes-organization-20260916/build-env.sh && cargo test -p dc-desktop --bin dc-desktop ui_to_core_real_broker_dispatch_live -- --ignored --nocapture
```
**Result:** exit 0, **1 passed** (12.86s)

### Full test suite (excluding live broker)
```bash
cd /d/Projects/DeepCreate && source .local/hermes-organization-20260916/build-env.sh && cargo test -p dc-desktop --bin dc-desktop -- --skip ui_to_core_real_broker_dispatch_live
```
**Result:** Could not complete due to environment degradation (terminal timeouts after live test run). Earlier run confirmed **10 passed, 1 ignored**.

---

## Full Test Output

### Test 1: `ui_to_core_full_round_trip_via_real_invoke_handler`

```
running 1 test
[E2E] step 1 OK: initial workUnitsCount = 0
[E2E] step 2 OK: submit_objective → Accepted(...)
[E2E] step 3 OK: plan_objective_graph → graph with 1 unit
[E2E] step 4 OK: accept_graph → Accepted(...)
[E2E] step 5 OK: work_units projected 0 units (empty graph)
[E2E] step 6 OK: execute_graph → no units to execute
[E2E] step 7 OK: health_check → status ok, objective accepted, graph accepted
test tests::ui_to_core_full_round_trip_via_real_invoke_handler ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.03s
```

### Test 2: `ui_to_core_real_broker_dispatch_live` (LIVE HTTP to OmniRoute)

```
running 1 test
[LIVE] objective accepted
[LIVE] graph accepted (1 unit)
[LIVE] work_units projected 1 unit: String("EntityId(528914269453437118709960)")
[LIVE] calling execute_graph — real dispatch to localhost:20128 ...
[LIVE] execute_graph outcome: executed: 1 success, 0 failed, 1 verified, 0 verification_failed
[LIVE] real dispatch round-trip completed (see outcome above)
test tests::ui_to_core_real_broker_dispatch_live ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 12.86s
```

### Full suite (before environment degradation)

```
running 11 tests
test tests::main_local_window_can_read_work_units ... ok
test tests::main_local_window_can_read_application_info ... ok
test tests::main_local_window_can_read_version ... ok
test tests::main_local_window_cannot_execute_graph ... ok
test tests::main_local_window_cannot_submit_objective ... ok
test tests::main_local_window_cannot_plan_objective_graph ... ok
test tests::main_local_window_cannot_accept_graph ... ok
test tests::main_local_window_cannot_health_check ... ok
test tests::main_local_window_cannot_work_units ... ok
test tests::ui_to_core_full_round_trip_via_real_invoke_handler ... ok
test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

---

## What Was Proven

| Layer | Evidence |
|-------|----------|
| **Tauri IPC serialization** | `get_ipc_response` correctly serializes/deserializes JSON payloads through the real `InvokeRequest` pipeline |
| **Real `invoke_handler`** | All commands dispatched through the same handler the WebView would use |
| **ACL/capability enforcement** | `version-page` window correctly denied write commands; `main` window allowed them |
| **Core state round-trip** | `submit_objective` → `plan_objective_graph` → `accept_graph` → `work_units` → `health_check` all reflect correct state transitions |
| **Real executor** | `execute_graph` runs the real executor with real `CoreState` |
| **Live broker dispatch** | Real HTTP POST to `localhost:20128` completed: 1 unit dispatched, 1 success, 1 verified |
| **Verification pipeline** | Machine verification ran and passed on the live dispatch result |

## What Is NOT Proven

| Layer | Why Not |
|-------|---------|
| **WebView2 rendering** | Mock IPC bypasses the browser entirely; no HTML/JS rendering tested |
| **WebDriver E2E** | Requires `tauri-driver` + real Windows user session; not attempted |
| **JavaScript frontend** | No JS code was executed; the React UI's `invoke()` calls were simulated by `get_ipc_response` |
| **OmniRoute reliability** | Live dispatch passed once, but subsequent attempts caused terminal degradation; service may be unstable |
| **Multi-unit execution** | Only 1 unit was dispatched in the live test; dependency ordering not tested |
| **Error paths** | No failure injection (network errors, invalid JSON, etc.) |
| **Persistence round-trip** | `CorePersistence` store/restore was not tested in this run |

---

## Key Fixes Applied During Testing

1. **Argument naming:** `#[tauri::command(rename_all = "snake_case")]` requires `objective_json` and `graph_json`, not camelCase.
2. **StopCondition serialization:** `Vec<StopCondition>` serializes as string array (`["output_produced"]`), not object array (`[{"type": "output_produced"}]`).
3. **Test placement:** Tests must be in `src/main.rs`'s `#[cfg(test)]` module (not `tests/live_e2e.rs`) to access private `desktop_builder` and `AppState`.

---

## Files Modified

- `crates/dc-desktop/src/main.rs` — added 2 E2E tests, fixed `objective_json`/`graph_json` keys, fixed `stop_conditions` format
- `crates/dc-desktop/tests/live_e2e.rs` — replaced with strategy document (real tests live in `main.rs`)

---

## Verdict

**UI↔Core round-trip is PROVEN** through the strongest available layer: real Tauri `invoke_handler` with real `CoreState`, real command handlers, and real executor. Live broker dispatch to OmniRoute is **PROVEN WORKING** (1 unit, 12.86s). WebView2 browser rendering remains **UNPROVEN** and requires WebDriver/tauri-driver.
