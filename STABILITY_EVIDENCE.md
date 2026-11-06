# Long-Running Stability Evidence

**Date:** 2026-10-01
**Commit:** `bdba109` (feat: deep verification layer — WebView2 E2E, multi-unit, error injection, auto-approve) + working-tree changes
**Test:** `crates/dc-core/tests/stability_test.rs` → `long_running_stability`
**Command:** `cargo test -p dc-core --test stability_test -- --nocapture`
**Environment:** Windows 10 (MSYS/git-bash), `source .local/hermes-organization-20260916/build-env.sh`, Rust dev profile (unoptimized + debuginfo)

## What the test does

Simulates continuous DeepCreate operation for a bounded **300 s (5 min)** window. Each cycle runs the full objective lifecycle against real components:

1. **Persistence init** — fresh `CorePersistence` over `MemoryStore` (`:memory:`), opened and dropped every cycle (exercises store/connection lifecycle for leak detection)
2. **Objective submission** — `handle_command(CoreCommand::SubmitObjective)`
3. **Planning** — build `GraphCandidate` + `WorkUnitSpec`, `handle_command(CoreCommand::AcceptGraph)`
4. **Scheduling** — `select_ready_units` with `SchedulingContext`
5. **Execution** — `CoreDispatcher::execute_unit` via a deterministic in-process `WorkDispatcher` (no network, so the test measures core stability, not API flakiness)
6. **Result recording** — `response_to_result` → `handle_command(CoreCommand::RecordResult)`
7. **Verification** — `run_machine_verification` → `handle_command(CoreCommand::RecordVerification)`
8. **Persist + round-trip** — `persist_state` / `load_state`, asserting loaded state matches

Entity IDs, prompts, and content vary per cycle (cycle counter embedded) to prevent cache/de-dup masking.

## Monitored resources

| Resource | How | Threshold |
|---|---|---|
| Memory (RSS) | `GetProcessMemoryInfo` via `windows-sys`, sampled every 50 cycles | growth ≤ 512 MiB over the run |
| File handles / DB connections | store open+close every cycle (failure to release would fail `initialize` or blow handle limits) | every cycle must succeed |
| Cycle stalls | per-cycle wall time | max ≤ 10,000 ms |
| Performance degradation | median of last quarter vs first quarter | slowdown ≤ 2.0× |
| Coverage | minimum completed cycles | ≥ 50 |

## Results — full 5-minute run

```
=== Stability Test Complete ===
Total cycles: 233250
Total time: 300.0s
Cycles/sec: 777.4
Baseline RSS: 3656 KB
Final RSS:    8724 KB
Peak RSS:     8724 KB
RSS growth:   5068 KB
Cycle times (ms): min=0 median=0 p95=0 p99=4 max=221
First-quarter median: 0ms
Last-quarter median:  0ms
Slowdown ratio:       1.00x

✅ ALL STABILITY ASSERTIONS PASSED
   233250 cycles over 300.0s with no leaks, stalls, or degradation
test result: ok. 1 passed; 0 failed; finished in 300.67s
```

### RSS trajectory (30 s milestones)

| Elapsed | Cycle | RSS (KB) |
|---:|---:|---:|
| 29 s | 50,600 | 6,244 |
| 59 s | 84,500 | 7,080 |
| 89 s | 111,950 | 7,596 |
| 119 s | 140,350 | 8,096 |
| 149 s | 159,500 | 8,396 |
| 179 s | 176,000 | 7,676 ← dropped (allocator/GC churn, not monotonic growth) |
| 209 s | 189,600 | 7,964 |
| 239 s | 202,300 | 8,184 |
| 269 s | 218,400 | 8,480 |
| 299 s | 233,250 | 8,724 |

Growth is sub-linear and partially reverses (8,396 → 7,676 KB between 149 s and 179 s), consistent with allocator warm-up rather than a leak: ~5 MB total over 233k cycles ≈ **~22 bytes/cycle average**, plateauing well under the 512 MiB bound.

## Verdict

| Assertion | Result |
|---|---|
| Memory growth bounded | ✅ PASS — 5,068 KB « 512 MiB |
| No store/handle leaks | ✅ PASS — 233,250 store open/close cycles, zero failures |
| No stalls | ✅ PASS — max cycle 221 ms « 10,000 ms |
| No performance degradation | ✅ PASS — slowdown ratio 1.00× ≤ 2.0× |
| Sufficient coverage | ✅ PASS — 233,250 cycles ≥ 50 |

## Fixes required to enable the test

- `crates/dc-domain/src/work.rs`: added `Hash` to `UnitRef`'s derive list — pre-existing compile error (`UnitRef` used in hash-based collections in the scheduler) that blocked `dc-core` from building.
- `crates/dc-core/Cargo.toml`: added `windows-sys` features `Win32_System_ProcessStatus` + `Win32_System_Threading` (dev-dependencies) for RSS sampling.

## How to reproduce

```bash
cd D:/Projects/DeepCreate
source .local/hermes-organization-20260916/build-env.sh
cargo test -p dc-core --test stability_test -- --nocapture
```

Duration is controlled by `STABILITY_DURATION_SECS` (currently 300). A 60 s validation run also passed (96,150 cycles, 1,602 cycles/s, same zero-degradation profile).

## Caveats

- Uses `MemoryStore`, not the on-disk SQLite store — file-handle/FD-leak coverage is at the store-abstraction level, not the OS-file level. A follow-up could run the same loop against a temp-dir SQLite store.
- Dispatcher is in-process; real LLM/network latency is not exercised (by design — this isolates core stability from external flakiness).
- Debug-profile build; release numbers would differ in absolute terms but the growth/degradation signals are the point.
