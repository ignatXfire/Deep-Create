# Rollback Procedure — Deep Create

This document describes how to roll back a Deep Create deployment when a
release misbehaves in production. It covers the application binary, the
SQLite store, and configuration.

## When to roll back

Roll back when any of the following hold after a deploy:

- `health_check` returns `status: "degraded"` for more than 5 minutes and
  the cause is the new release (not an external outage).
- Structured logs show a sustained increase in `errors_total` or
  `dispatch_failed` events immediately after the deploy.
- The application fails to start (panic on init, persistence migration
  failure).
- Owner-observable data corruption in the work graph.

Do **not** roll back for: transient broker unreachability (the `broker`
field in `health_check` reports this but does not degrade overall status),
single failed dispatches that succeed on retry.

## Prerequisites

Before deploying, ensure you have:

1. **The previous release binary** — tagged and stored. Builds are
   reproducible from git: `git checkout <previous-tag> && cargo build --release -p dc-desktop`.
2. **A store backup** — the SQLite store is the single source of truth.
   Back it up *before* every deploy:
   ```bash
   # Stop the app first (WAL checkpoint happens on clean shutdown)
   cp "$DC_STORE_PATH" "$DC_STORE_PATH.backup-$(date +%Y%m%d-%H%M%S)"
   ```
3. **The previous configuration** — environment variables (`DC_STORE_PATH`,
   `DC_LOG`, `DC_LOG_FORMAT`) and any endpoint config.

## Rollback steps

### 1. Stop the current release

Graceful shutdown (preferred — lets the store checkpoint):

- **Desktop**: close the window. Tauri exits the event loop cleanly.
- **Console/headless**: send SIGINT (Ctrl+C, exit code 130) or
  SIGTERM-equivalent (Ctrl+Break, exit code 143). The process logs a
  `shutdown` event to structured logs.

If the process is hung, kill it. SQLite is in WAL mode, so committed
transactions survive an unclean kill; uncommitted transactions are rolled
back automatically on next open.

### 2. Verify store integrity

Before restoring any backup, check whether the current store is intact:

```bash
sqlite3 "$DC_STORE_PATH" "PRAGMA integrity_check;"
```

- If `ok` → the store is fine; skip to step 4 (binary rollback only).
- If corruption is reported → restore from the pre-deploy backup:
  ```bash
  cp "$DC_STORE_PATH.backup-<timestamp>" "$DC_STORE_PATH"
  # Also remove stale WAL/SHM from the corrupted copy
  rm -f "$DC_STORE_PATH-wal" "$DC_STORE_PATH-shm"
  ```

### 3. Verify state compatibility

The previous release must be able to read the store schema. Schema
migrations are additive within a release series; if the new release ran a
migration the old one cannot read, you MUST restore the pre-deploy backup
(step 2) — rolling back the binary alone is not sufficient.

### 4. Restore the previous binary

```bash
git checkout <previous-tag>
cargo build --release -p dc-desktop
# deploy the binary per your normal process
```

### 5. Restart and verify

Start the application and confirm:

```bash
# Via the UI or IPC — health_check should return:
#   status: "ok"
#   store: "reachable"
#   broker: "reachable" (or "unreachable" if the model endpoint is down —
#         this does not affect app health)
```

Check structured logs for a clean `startup` event with no `error`-level
entries in the first 60 seconds.

### 6. Post-rollback

- Record the incident: what failed, which metrics/logs showed it, and why
  the rollback was needed.
- Keep the failed release's binary and logs for diagnosis.
- Do not re-deploy until the root cause is fixed and verified against the
  failing scenario.

## Quick reference

| Situation | Action |
|---|---|
| New binary panics on start | Roll back binary only |
| Store corrupted after deploy | Restore backup + roll back binary |
| Broker endpoint down | No rollback — external; wait or fix endpoint |
| Elevated `errors_total` after deploy | Investigate logs; roll back if caused by release |
| `health_check` `degraded` >5 min | Check `store` field; roll back if store unreachable |

## Signals cheat sheet

| Signal | Where | Meaning |
|---|---|---|
| `shutdown` log event | structured logs | graceful shutdown initiated |
| `dispatch_failed` | structured logs | all broker retries exhausted |
| `errors_total` | `health_check.metrics` | cumulative error counter |
| `uptime_secs` | `health_check.metrics` | process uptime — low value after deploy means crash-looping |
