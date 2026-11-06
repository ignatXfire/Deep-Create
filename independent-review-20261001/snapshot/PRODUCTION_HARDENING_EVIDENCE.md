# Production Hardening — Evidence

**Date:** 2026-10-01 (UTC)
**Branch:** `codex/deep-create-foundation`
**Base commit:** `e19936d` (with concurrent unrelated working-tree changes from other sessions left untouched)
**Scope:** Structured logging, health check, graceful shutdown, rollback procedure, metrics counters.

---

## 1. Structured logging (tracing)

**Crates instrumented:** `dc-core`, `dc-broker`, `dc-desktop`.

**Implementation:**
- `tracing` + `tracing-subscriber` added as workspace dependencies (`Cargo.toml` `[workspace.dependencies]`).
- `dc-desktop::main()` calls `init_tracing()` at startup: `EnvFilter` from `DC_LOG` env var (default `info`), human-readable output to stderr; `DC_LOG_FORMAT=json` switches to JSON logs.

**Key events emitted:**

| Event | Crate | Site | Level |
|---|---|---|---|
| `objective_submitted` | dc-core | `handle_command` → `AcceptObjective` | INFO |
| `graph_accepted` | dc-core | `handle_command` → `AcceptGraph` | INFO |
| `unit_dispatched` | dc-core | `handle_command` → `StartAttempt` | INFO |
| `verification_completed` | dc-core | `handle_command` → `RecordVerification` | INFO |
| `unit_completed` | dc-core | `handle_command` → `ApplyVerifiedResult` | INFO |
| `command rejected` | dc-core | `handle_command` → `decide_transition` error | WARN |
| `dispatch_started` | dc-broker | `dispatch_work_unit` entry | INFO |
| `dispatch_succeeded` | dc-broker | HTTP 200 path | INFO |
| `dispatch_refused` | dc-broker | HTTP 400 path | WARN |
| `dispatch_retry` | dc-broker | retry loop backoff | WARN |
| `dispatch_failed` | dc-broker | retries exhausted | ERROR |
| `startup` / `logging_initialized` / `shutdown` | dc-desktop | `main()` / signal handler | INFO |
| `health_store_unreachable` | dc-desktop | `health_check` | WARN |

**Proof:** test `dc-core::command::tests::tracing_events_emitted_on_objective_accept`
installs a buffer-capturing subscriber, issues `AcceptObjective`, and asserts
the emitted output contains `objective_submitted`. **Result: PASS.**

```
test command::tests::tracing_events_emitted_on_objective_accept ... ok
```

## 2. Health check command

`health_check` Tauri command in `dc-desktop` rewritten. Returns JSON:

```json
{
  "status": "ok" | "degraded",
  "version": "0.1.0",
  "store": "reachable" | "unreachable" | "not_configured",
  "broker": "reachable" | "unreachable" | "misconfigured",
  "active_jobs": <units in Running stage>,
  "has_objective": <bool>,
  "units": <count>, "verifications": <count>,
  "metrics": { …see §5… },
  "timestamp": "<rfc3339>"
}
```

- **Store reachable**: opens the SQLite file read-only via rusqlite and runs `SELECT 1`.
- **Broker reachable**: TCP connect (500 ms timeout) to the host:port parsed from `EndpointConfig::base_url`.
- **Degraded semantics**: `status = "degraded"` only when the configured store is unreachable; broker is external/transient and reported without degrading the app.

**Proof:** E2E test `ui_to_core_full_round_trip_via_real_invoke_handler` invokes
`health_check` over the real Tauri IPC handler and asserts `status == "ok"`.
**Result: PASS** (10/10 dc-desktop tests pass, including the health-check E2E).

## 3. Graceful shutdown

- **Windows** (`dc-desktop::install_signal_handler`, `#[cfg(windows)]`): `SetConsoleCtrlHandler` handles `CTRL_C_EVENT` (SIGINT → exit 130) and `CTRL_BREAK_EVENT` (SIGTERM-equivalent → exit 143). Handler logs a `shutdown` event with the signal name before exiting. The workspace `unsafe_code = deny` lint is explicitly overridden with `#[allow(unsafe_code)]` on this single function (required for the Win32 FFI call).
- **Other platforms**: no-op fallback with a debug log; Tauri's event loop handles window close.
- **Desktop GUI**: Tauri's default close behavior destroys windows and exits the event loop cleanly, which checkpoints the WAL-mode SQLite store.

## 4. Rollback procedure

Created `docs/deployment/ROLLBACK.md`: when to roll back (and when not to),
prerequisites (binary + store backup + config), 6-step procedure (stop →
verify store integrity → verify schema compatibility → restore binary →
restart and verify via `health_check` → post-rollback incident record),
quick-reference table, and a signals cheat sheet tied to the new metrics.

## 5. Metrics counters

New module `crates/dc-core/src/metrics.rs`: process-local lock-free counters
(`AtomicU64`) on a global `Metrics` registry (`Metrics::global()`).

Counters:
- `objectives_processed`
- `graphs_accepted`
- `units_dispatched`
- `units_completed`
- `verifications_passed` / `verifications_failed`
- `errors_total`
- `uptime_secs` (derived from `Instant::now()` at first access)

`Metrics::snapshot()` serializes all counters to JSON; exposed in
`health_check.metrics`. Instrumentation sites: `handle_command`
(objective/graph/dispatch/verification/completion/error paths).

**Proof:**
- `dc-core::metrics::tests::counters_increment` — counter arithmetic. PASS.
- `dc-core::metrics::tests::snapshot_has_all_fields` — snapshot shape. PASS.
- `dc-core::command::tests::metrics_increment_on_command_lifecycle` — asserts
  `objectives_processed` +1 after a successful `AcceptObjective` and
  `errors_total` +1 after a rejected command against the live global registry. PASS.

## 6. Verification results

Build: `cargo check --workspace` — **clean** (one pre-existing unused-import
warning in `dc-core::scheduler`, untouched).

Tests (2026-10-01, sequential run to avoid Windows link.exe lock contention):

| Crate | Result |
|---|---|
| dc-core (lib) | 67 passed, 0 failed |
| dc-broker | 6+6+4 passed, 0 failed |
| dc-desktop (bin, incl. E2E) | 10 passed, 0 failed, 2 ignored |
| dc-watchdog | 4 passed, 0 failed (when run separately) |
| Full workspace | all green except one flaky `dc-watchdog::anchor_anti_rollback_refused` failure under heavy parallel load — passes in isolation both **before and after** this change (`git stash` verified pre-existing flakiness); dc-watchdog was not modified. |

## Files changed

- `Cargo.toml` — workspace deps: `tracing`, `tracing-subscriber`
- `crates/dc-core/Cargo.toml` — `tracing` dep, `tracing-subscriber` dev-dep
- `crates/dc-core/src/lib.rs` — `pub mod metrics;`
- `crates/dc-core/src/metrics.rs` — **new**: metrics registry
- `crates/dc-core/src/command.rs` — tracing + metrics instrumentation, `command_kind_name`, 2 new tests
- `crates/dc-broker/Cargo.toml` — `tracing` dep
- `crates/dc-broker/src/dispatch.rs` — dispatch lifecycle logging
- `crates/dc-desktop/Cargo.toml` — `tracing`, `tracing-subscriber`, `rusqlite`, `windows-sys` (Win32_System_Console)
- `crates/dc-desktop/src/main.rs` — `init_tracing()`, rewritten `health_check`, store/broker reachability probes, `install_signal_handler`, `AppState` extended with `store_path` + `broker_endpoint`
- `docs/deployment/ROLLBACK.md` — **new**: rollback procedure
- `PRODUCTION_HARDENING_EVIDENCE.md` — **new**: this file

## Known limitations / follow-ups

- Metrics are process-local; they reset on restart. Persisting them to the store is future work.
- Graceful shutdown on Windows exits the process from the console handler (destructors not guaranteed to run); WAL-mode SQLite keeps committed data durable.
- Broker reachability is a TCP-level check, not an authenticated API check.
