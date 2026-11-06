# Extended Stability Soak Test Evidence (30 minutes)

## Run Metadata

- **Repository**: D:/Projects/DeepCreate
- **Branch**: codex/deep-create-foundation
- **Commit**: c1d1058
- **Test**: `crates/dc-core/tests/extended_stability_soak.rs` (`extended_stability_soak_30min`)
- **Duration**: 1800.4 s (30.0 minutes)
- **Total cycles**: 1,780,000
- **Overall throughput**: 988.65 cycles/sec
- **Errors / crashes**: 0
- **Cargo test result**: `test result: ok. 1 passed`

## Memory (RSS) Time Series

Sampled every 60 seconds by the test. Values in KB / MB.

| Elapsed (s) | RSS (KB) | RSS (MB) |
|-------------|----------|----------|
| 0    |  3,944 |  3.85 |
| 60   |  7,084 |  6.92 |
| 120  |  8,804 |  8.60 |
| 180  | 10,328 | 10.09 |
| 240  | 11,732 | 11.46 |
| 300  | 13,376 | 13.06 |
| 360  | 14,848 | 14.50 |
| 420  | 15,792 | 15.42 |
| 480  | 16,560 | 16.17 |
| 540  | 17,328 | 16.92 |
| 601  | 18,048 | 17.62 |
| 661  | 18,752 | 18.31 |
| 721  | 19,444 | 18.99 |
| 781  | 20,276 | 19.80 |
| 841  | 21,236 | 20.74 |
| 901  | 22,160 | 21.64 |
| 961  | 23,184 | 22.64 |
| 1021 | 23,888 | 23.33 |
| 1081 | 24,288 | 23.72 |
| 1142 | 24,992 | 24.41 |
| 1202 | 25,888 | 25.28 |
| 1262 | 26,720 | 26.09 |
| 1322 | 27,572 | 26.93 |
| 1382 | 28,384 | 27.72 |
| 1442 | 29,024 | 28.34 |
| 1503 | 29,556 | 28.86 |
| 1564 | 30,068 | 29.36 |
| 1624 | 30,580 | 29.86 |
| 1684 | 31,028 | 30.30 |
| 1744 | 31,668 | 30.93 |
| 1800 | 32,436 | 31.68 |

**RSS analysis**

- Baseline: 3,944 KB → Final: 32,436 KB → Peak: 32,436 KB
- Total growth: 28,492 KB (27.8 MB) over 30 min
- Linear fit (per-minute samples, R^2 = 0.9830): slope ≈ **821 KB/min**, intercept ≈ 9,898 KB
- RSS is **NOT plateauing** — it grows approximately linearly throughout the run
- Growth rate per 5-min window is mildly decelerating but does not approach zero:
  - 0-5 min: +6,288 KB
  - 5-10 min: +3,196 KB
  - 10-15 min: +3,408 KB
  - 15-20 min: +2,704 KB
  - 20-25 min: +2,900 KB
  - 25-30 min: +2,368 KB
- Projected at this slope: ~59 MB at 60 min, ~108 MB at 120 min. Bounded for any realistic session, but this is a real per-cycle allocation (~16 KB/cycle retained) — likely `MemoryStore` state accumulated by the test harness, not a leak in production persistence.

## Throughput (cycles/sec) per 5-minute Window

| Window    | Cycles   | Cycles/sec |
|-----------|----------|------------|
| 0-5 min   | 540,661  | 1,802.20   |
| 5-10 min  | 298,340  |   994.47   |
| 10-15 min | 261,666  |   872.22   |
| 15-20 min | 258,073  |   860.24   |
| 20-25 min | 236,288  |   787.63   |
| 25-30 min | 184,972  |   616.57   |

**Throughput analysis**

- Mean: 993 c/s, StdDev: 430 c/s, CV: 0.433 (high variance)
- First 15 min avg: 1,224.3 c/s
- Last 15 min avg: 761.7 c/s
- **Second-half / first-half ratio: 0.62x** — throughput declined ~38% over the run
- Per-minute throughput peaked at 2,347 c/s in minute 0 and fell to ~520-920 c/s in minutes 25-29
- The in-test assertion `slowdown_ratio = 1.00x` is **misleading**: it is computed from the *median cycle time* of the first vs last quartile, but the median cycle time is 0 ms in both quartiles (sub-millisecond cycles dominate). The per-window throughput data above is the honest signal.

## Cycle Time Statistics

- Min: 0 ms
- Median: 0 ms
- P95: 0 ms
- P99: 3 ms
- Max: 208 ms
- First-quartile median: 0 ms
- Last-quartile median: 0 ms

P99 is 3 ms and max is 208 ms (well under the 10 s stall threshold) — no individual cycle stalls. The throughput decline is *distributed* (many slightly slower cycles), not stall-driven.

## Errors

- Total errors encountered: **0**
- No panics, no failed assertions, no crashes. Cargo test process exited 0.

## Verdict

**PASS WITH OBSERVATIONS — no errors, stalls, or failures over 30 minutes; throughput declines ~38% and RSS grows linearly (~821 KB/min) — both worth follow-up before claiming production-grade long-run stability.**

- Throughput stable: **false** — 0.62x second-half/first-half ratio, monotonic decline across 5-min windows
- RSS plateauing: **false** — linear growth at ~821 KB/min, R^2 = 0.983 (well-fit by a line, not a plateau)
- Errors: **0** (hard requirement met)
- Stalls (max cycle < 10 s): **pass** (max 208 ms)
- Memory growth < 1 GiB test cap: **pass** (28 MB)
- The in-test `slowdown_ratio` assertion is **not sensitive enough** for sub-millisecond workloads; recommend replacing with a windowed-throughput check

### Recommended follow-ups

1. Investigate the ~16 KB/cycle retained allocation — most likely `MemoryStore` growth in the test harness, not production code. Confirm by re-running with a real persistent backend.
2. Tighten the slowdown assertion: use first-window vs last-window cycles/sec with a threshold (e.g. ≥ 0.9x), not medians of cycle times when median ≈ 0.
3. Extend to a 2-hour soak before shipping; at the current slope RSS would reach ~108 MB — fine, but the trend should be confirmed to flatten (e.g. due to GC / allocator equilibrium) rather than continue indefinitely.
