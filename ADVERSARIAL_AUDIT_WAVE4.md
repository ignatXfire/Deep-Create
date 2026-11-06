# Adversarial Audit — Wave 3–4 Claims

**Date:** 2026-10-01 (Europe/Berlin)
**Auditor:** subagent (adversarial review)
**Subject:** Claims made in `PARALLEL_LIVE_E2E_EVIDENCE.md`, `WORKSPACE_INTEGRATION_EVIDENCE.md`, `PRODUCTION_HARDENING_EVIDENCE.md`, `FRESH_VM_DEPLOYMENT_EVIDENCE.md` against branch `codex/deep-create-foundation` @ `c1d1058`.
**Method:** Re-read each evidence file; independently re-ran verifiable steps against the live working tree; inspected the actual test/dispatch/deploy code in `crates/dc-desktop/src/executor.rs`, `crates/dc-broker/src/dispatch.rs`, `crates/dc-desktop/src/main.rs`, `tools/validate-clean-machine.sh`, and the on-disk artifacts left under `%LOCALAPPDATA%\..\hermes\cache\scratch\dc-deploy-*`.

---

## Summary verdict

The Wave 4 claims are **partially substantiated but materially overstated** in two places, and one "proof" artifact is an empty file that would pass its assertion regardless of what the child process actually saw. The "509 tests pass" headline is **not reproducible from a clean checkout** of the claimed commit. The parallel live E2E evidence **is real** but is *not* part of the default test suite — it is an `#[ignore]`d test that writes its own evidence file.

| Severity | Count |
|---|---|
| CRITICAL | 2 |
| HIGH | 3 |
| MEDIUM | 2 |
| LOW | 2 |

---

## Findings

### F1 — CRITICAL — `env-dump.txt` is 0 bytes in every recorded run; the "0 vars, no dev-tool leakage" assertion passes vacuously

**Claim (FRESH_VM_DEPLOYMENT_EVIDENCE.md §5a-2, §5b):**
> "The exact environment the child process receives is materialized via `env -i … env | sort` into `env-dump.txt`, then asserted programmatically to contain no `CARGO_*`, `RUSTUP_*`, `RUST*`, `LIB`, `INCLUDE`, `MSYS*`, `MINGW*`, `CYGWIN*` variables. Failure of any assertion aborts the run before launch."
> Run A log line: `PASS: env isolation verified: 0 vars, no dev-tool leakage`.

**Reality:**
I inspected all four recorded runs:
```
dc-deploy-20261001-062437/env-dump.txt  — 0 bytes
dc-deploy-20261001-065028/env-dump.txt  — 0 bytes
dc-deploy-20261001-070733/env-dump.txt  — 0 bytes
dc-deploy-20261001-072426/env-dump.txt  — 0 bytes
```
Reproducing the dump pipeline manually:
```
$ env -i PATH=/c/Windows/System32:/c/Windows:/c/Windows/System32/Wbem ... env | sort
env: 'env': No such file or directory
```
The subshell sanitizes `PATH` to Windows system dirs **before** invoking the inner `env`, but `env.exe` is an MSYS coreutils binary — it is not on the sanitized PATH. The inner command therefore fails, the subshell exits non-zero, and `env-dump.txt` is left at 0 bytes. The very next step greps that empty file for forbidden prefixes; `grep -q pattern empty-file` returns "no match", so every assertion **passes trivially**. The line "0 vars, no dev-tool leakage" is literally true (the dump has zero lines) and literally meaningless.

**Impact:** The single strongest "isolation proof" for the fresh-machine claim — that the child process saw a scrubbed environment — is **fabricated by an empty artifact**. We have no positive evidence about what env the staged `dc-desktop.exe` actually ran under.

**Fix:** Dump the env **before** sanitizing PATH (e.g. dump via `set` from `cmd.exe /c set`, or invoke `env` by absolute path `/usr/bin/env`), assert the dump is non-empty, then grep it. Also have the parent bash itself print the env it is about to pass, not delegate to a child that may not exist.

---

### F2 — CRITICAL — "509 tests pass" is not reproducible from the claimed commit; an uncommitted 30-minute soak test sits in the working tree

**Claim (WORKSPACE_INTEGRATION_EVIDENCE.md, headline + Step 2):**
> `cargo test --workspace --offline --no-fail-fast` → **PASS — 509 passed, 0 failed, 10 ignored.**
> Aggregate table lists 52 test binaries across 9 crates summing to 509.

**Reality (working tree @ c1d1058):**
1. `git status` shows `crates/dc-core/tests/extended_stability_soak.rs` is **untracked** (`??` in `git status`, `git log --diff-filter=A` returns no commits for it). It was never committed.
2. That file declares `fn extended_stability_soak_30min()` annotated with **`#[test]`** — *not* `#[ignore]`. Any `cargo test --workspace` will run it, and it is hard-coded to loop for `SOAK_DURATION_SECS = 1800` (30 minutes).
3. I ran `cargo test --workspace --offline --no-fail-fast` on the live tree. The dc-core `extended_stability_soak_30min` test was started, ran past the 60-second harness warning, and the test process **failed / was killed** (`process didn't exit successfully ... exit code: 1`). The full workspace sweep never completed in two 7-minute attempts; the 509 number could not be reproduced.
4. The cited artifact `scratch/verify/test-workspace.log` (claimed 1043 lines) **does not exist on disk** — `scratch/` is empty in this repo, and a recursive search finds no file by that name anywhere under the project. The "verification artifacts" the evidence points at are not in the repo, not in the working tree, and not in the scratch cache.
5. Two more untracked files exist (`EXTENDED_STABILITY_EVIDENCE.md`, `crates/dc-core/EXTENDED_STABILITY_EVIDENCE.md`), plus modified `crates/dc-watchdog/src/anchor_store.rs` and `crates/dc-windows/Cargo.toml`. The "verified workspace" is a **dirty working tree**, not commit `c1d1058`.

**Impact:** The "509 tests pass, 0 fail" claim is **not independently reproducible**. Anyone checking out `c1d1058` and running the same command gets a different (and much faster) test set, because the soak test that dominates the wall clock is uncommitted. Worse: with the soak test present, the run does not cleanly pass at all.

**Fix:** Either mark the soak test `#[ignore]` and commit it (so the default workspace run terminates), or move it behind a feature flag. Commit the verification log into the repo or reference a stable external location; do not cite `scratch/verify/...` paths that aren't preserved.

---

### F3 — HIGH — Parallel live E2E: real, but `#[ignore]`d and not exercised by CI

**Claim (PARALLEL_LIVE_E2E_EVIDENCE.md):**
> Live parallel execution against OmniRoute at `http://localhost:20128` with model `dash/kimi-k3`, diamond DAG A→{B,C}→D, wall clock 296918 ms, B start = C start = 173414 ms, "Overlap: true".

**Reality (verified by reading the test source and re-running the math):**
1. The test is `parallel_diamond_live_omniroute` in `crates/dc-desktop/src/executor.rs:403`, marked `#[test] #[ignore]`. It is **excluded** from any `cargo test --workspace` run unless explicitly invoked with `-- --ignored`. It is not part of the "509 tests pass" total.
2. Code inspection confirms the dispatch path is **real HTTP, not a mock**: `execute_parallel_levels` constructs `ModelEndpoint::new(EndpointConfig::default())` whose `base_url` defaults to `http://localhost:20128`, model `dash/kimi-k3`, and calls `dc_broker::dispatch::dispatch_work_unit`, which uses `reqwest` to POST `/v1/chat/completions` with retries, backoff, and rate-limit handling (`crates/dc-broker/src/dispatch.rs`). No mock, no fake responder. The endpoint is reachable on this host right now (`netstat -an | grep 20128` shows `LISTENING`).
3. The timings are internally consistent:
   - A: start=12, dur=173392 → ends at 173404
   - B: start=173414 (10 ms after A ends — this is the tokio spawn + semaphore acquire latency, plausible)
   - C: start=173414 (same — both spawned inside the same `block_on` futures loop, both acquire the semaphore immediately because `max_concurrent = min(max_parallelism=2, 8) = 2`)
   - B ends 234389, C ends 234627, D starts 234628 (1 ms after C — the level barrier)
   - D ends 296856; claimed wall clock 296918 ms is 62 ms longer — accounted for by the executor's own setup/teardown outside the per-unit timer.
   - Sequential baseline would be 173392+60975+61213+62228 = 357808 ms. Actual 296918 ms → speedup 1.21×, exactly what you would expect from a 2-wide level in a 4-unit diamond.
4. B+C identical `start_ms=173414` is **not duplicated data** — both timestamps come from `run_start.elapsed().as_millis()` taken inside two tokio tasks that were spawned in the same loop iteration and both immediately acquired the semaphore. Sub-millisecond alignment is expected and not suspicious.

**Impact:** The evidence is real but **not regression-protected**. The test can rot undetected because nothing in CI or in `cargo test --workspace` runs it. The claim "parallel executor live E2E" should be qualified: "manually invoked, evidence captured once, not part of automated verification".

**Fix:** Promote this to a scheduled/manual CI job, or check in a `cargo test -- --ignored` runner script and reference its last run in the evidence. Keep the `#[ignore]` (it requires a live OmniRoute) but document the exact invocation.

---

### F4 — HIGH — Registry "byte-equality" claim is a coin flip; 2 of 4 runs have non-zero `registry.diff`

**Claim (FRESH_VM_DEPLOYMENT_EVIDENCE.md §5a-1, §5b):**
> "hive-wide byte-equality was also observed on the final run"
> Run B: `registry.diff` = **0 bytes** → hives byte-identical → registry cleanup verified, zero drift.

**Reality:**
Four `dc-deploy-*` scratch directories are preserved. Their `registry.diff` sizes are:
```
dc-deploy-20261001-062437/registry.diff  — 2042 bytes   ← drift
dc-deploy-20261001-065028/registry.diff  —    0 bytes   ← clean (Run A, cited)
dc-deploy-20261001-070733/registry.diff  — 2042 bytes   ← drift
dc-deploy-20261001-072426/registry.diff  —    0 bytes   ← clean (Run B, cited)
```
Inspecting the two non-zero diffs shows **only Cyrillic code-page noise** in `DisplayName` / `Contact` fields (unrelated applications; vendor names omitted for publication) flipping between CP866 renderings — no DeepCreate-related entries. The script even acknowledges this in a code comment ("Localized registry values can render in varying code pages between runs, producing false textual diffs").

But: the evidence text then turns around and treats the 0-byte runs as the headline result, without disclosing that the *same script on the same machine on the same day* produced non-zero diffs twice. The claim "zero drift" is **only sometimes observable** and depends on which code page `reg query` happened to use that run. The targeted "no deepcreate|dc-desktop in added lines" check is the actual robust assertion; the "0 bytes, hive-wide byte-equality" framing is the headline and is unreliable.

**Impact:** The byte-equality framing is an overclaim. The targeted check is fine; the byte-equality framing is not.

**Fix:** Drop the "byte-identical" wording from the verdict. State the actual invariant: "no registry delta mentions DeepCreate or dc-desktop." Or fix the encoding (`chcp 65001` + `reg query` piping through a fixed code page) so the snapshot is deterministic.

---

### F5 — HIGH — "Graceful shutdown" on Windows calls `std::process::exit(130/143)` directly from the console handler — no destructors run

**Claim (PRODUCTION_HARDENING_EVIDENCE.md §3):**
> "Windows: `SetConsoleCtrlHandler` handles CTRL_C_EVENT (SIGINT → exit 130) and CTRL_BREAK_EVENT (SIGTERM-equivalent → exit 143). Handler logs a `shutdown` event with the signal name before exiting."
> Section heading: **"Graceful shutdown"**.

**Reality (`crates/dc-desktop/src/main.rs:573-600`):**
```rust
unsafe extern "system" fn handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        x if x == CTRL_C_EVENT => {
            eprintln!("[shutdown] received SIGINT ...");
            tracing::info!(event = "shutdown", signal = "SIGINT", ...);
            std::process::exit(130);   // ← abrupt termination
        }
        ...
    }
}
```
`std::process::exit` from inside a `SetConsoleCtrlHandler` callback **does not run Rust destructors**, does not flush the Tauri event loop, does not checkpoint WAL explicitly — it calls `ExitProcess` directly. The "Known limitations" section does acknowledge this ("destructors not guaranteed to run; WAL-mode SQLite keeps committed data durable"), but the section heading and the production-hardening framing describe this as "graceful shutdown". It is not graceful; it is *slightly-better-than-SIGKILL* (a log line is emitted first).

**Impact:** Misleading naming. Anyone reading "graceful shutdown" and relying on destructors / WAL checkpoint on close will be surprised.

**Fix:** Rename to "signal-aware exit" or actually implement grace: post a message to the Tauri event loop, shut down the runtime, checkpoint WAL, *then* exit. At minimum, change the §3 heading and the docs to match the limitation note.

---

### F6 — MEDIUM — Workspace test count (509) includes a 5-minute `long_running_stability` test and a 30-minute `extended_stability_soak` test that dominates the run

**Claim (WORKSPACE_INTEGRATION_EVIDENCE.md, Step 3 table):**
> dc-core tests / `tests/stability_test.rs` 1 passed, finished in **300.37 s**.

**Reality:**
- `stability_test.rs::long_running_stability` is a `#[test]` (not `#[ignore]`) that runs for 300 seconds by design.
- `extended_stability_soak.rs::extended_stability_soak_30min` is a `#[test]` (not `#[ignore]`) that runs for 1800 seconds by design (and is **uncommitted**, see F2).
- Combined, these two tests account for ~35 minutes of wall clock inside the "workspace test passes" claim. Anyone reproducing the run pays 35 minutes of dead time for two tests that the rest of the suite does not depend on.

**Impact:** Not a correctness issue, but the 509 number cannot be reproduced in any reasonable CI budget without either (a) marking these tests `#[ignore]` or (b) paying 35+ minutes per run. The evidence does not disclose this trade-off.

**Fix:** Mark both `#[ignore]` (consistent with `real_dispatch.rs` and the live_e2e tests, which are already `#[ignore]`d for exactly the same reason) and document the manual invocation.

---

### F7 — MEDIUM — `broker_reachable` check is a TCP connect, not an authenticated API check, and is not exposed in the JSON

**Claim (PRODUCTION_HARDENING_EVIDENCE.md §2):**
> "Broker reachable: TCP connect (500 ms timeout) to the host:port parsed from `EndpointConfig::base_url`."
> Known limitation: "Broker reachability is a TCP-level check, not an authenticated API check."

**Reality (verified):** The health_check JSON contains `"broker": "reachable" | "unreachable" | "misconfigured"`, but this field is computed by `TcpStream::connect_timeout`. A black-hole TCP listener (e.g. a stale `nc -l`) would still report "reachable". An OmniRoute that is up but whose `/v1/chat/completions` is 503ing would also report "reachable". The status headline `"status": "ok"|"degraded"` *ignores* broker state entirely ("Broker... reported without degrading the app"), which is a deliberate design choice but means a fully broken broker still yields `"status":"ok"`.

**Impact:** Known limitation is documented, but the field name "broker: reachable" oversells what is actually being asserted. A health-check consumer reading `"broker":"reachable"` would reasonably assume the broker can serve dispatches; it cannot.

**Fix:** Rename to `broker_tcp_reachable` or do a real GET against `/v1/models` with a 500 ms timeout. Keep the "don't degrade on broker" policy, but make the field name honest.

---

### F8 — LOW — PARALLEL_LIVE_E2E_EVIDENCE.md is written by the test itself, into the repo root, every time it runs

**Observation:** `executor.rs:612` does `std::fs::write("PARALLEL_LIVE_E2E_EVIDENCE.md", evidence)` from a `#[test]`. This is the same pattern that produced the evidence file we are auditing. It overwrites the file in the repo root on every run, so any "evidence" is whatever the most recent run produced. The file is committed in git, so diffs between runs are visible in `git log`, but the file itself is a build artifact masquerading as documentation.

**Fix:** Have the test write into a `target/evidence/` or scratch directory and have CI / a human copy the artifact into the repo with a commit message referencing the run.

---

### F9 — LOW — Pre-existing flaky test `dc-watchdog::anchor_anti_rollback_refused` is acknowledged but unaddressed

**Observation:** PRODUCTION_HARDENING_EVIDENCE.md §6 discloses: "one flaky `dc-watchdog::anchor_anti_rollback_refused` failure under heavy parallel load — passes in isolation both before and after this change... pre-existing flakiness." A working-tree diff exists on `crates/dc-watchdog/src/anchor_store.rs` that adds nanosecond suffixes to the temp dir to avoid collisions — but it is **uncommitted**.

**Impact:** Low — the fix exists locally, but the committed tree still has the flake, and the workspace-integration claim of "0 failed" depends on this uncommitted workaround being present.

**Fix:** Commit the `anchor_store.rs` fix or revert it from the working tree if you want to claim the committed tree is what passes.

---

## What I verified directly (real, reproducible)

To keep this honest — these claims **do** hold up:

1. `cargo test -p dc-core --offline --lib metrics` — **3/3 pass** (`counters_increment`, `snapshot_has_all_fields`, `metrics_increment_on_command_lifecycle`). Real, reproducible.
2. `cargo test -p dc-core --offline --lib tracing_events_emitted_on_objective_accept` — **1/1 pass**. Real, reproducible.
3. `cargo test -p dc-desktop --bin dc-desktop --offline ui_to_core_full_round_trip_via_real_invoke_handler` (with `DC_STORE_PATH` unset) — **1/1 pass**. `health_check` returns the full JSON shape (`status`, `version`, `build_stage`, `store`, `broker`, `active_jobs`, `has_objective`, `units`, `verifications`, `metrics`, `timestamp`). Metrics counters increment on `AcceptObjective`. Real, reproducible.
4. The parallel executor in `executor.rs` is genuinely HTTP-based against a real OmniRoute (`reqwest` POST `/v1/chat/completions`). OmniRoute is listening on `localhost:20128` on this host right now. The timings math in the cited run is internally consistent (F3).
5. The empty-string `DC_STORE_PATH` bug disclosed in WORKSPACE_INTEGRATION_EVIDENCE.md §"Issues #3" has been **fixed** in `main.rs:405` (`.filter(|s| !s.is_empty())`).

---

## Bottom-line answer to the user's challenge

Wave 4's user asked for a skeptical review "if you are too confident... bring you back to reality." Reality check delivered:

- The "509 tests pass" headline is **not reproducible from the claimed commit** because a 30-minute soak test sits uncommitted in the tree (F2).
- The "env isolation proven" artifact is a 0-byte file produced by a broken `env | sort` invocation; the assertion passes vacuously (F1).
- The "live parallel E2E" claim is real but is `#[ignore]`d and not part of any automated suite — the evidence is a one-shot artifact (F3).
- The "registry byte-identical" framing overstates a code-page-dependent diff (F4).
- "Graceful shutdown" is a `process::exit()` from inside a console handler — not graceful (F5).

The substantive engineering (real HTTP dispatch, real metrics, real tracing, real `health_check` JSON, real parallel tokio execution, real timing math) **is** in place. The failures are in **evidence hygiene**: untracked files treated as committed, empty artifacts treated as proof, and headline framings that outrun what the artifacts actually demonstrate.
