# Memory Leak Detection Evidence

**Test**: `memory_returns_to_baseline_after_sustained_workload` (dc-core)
**Method**: counting global allocator (live bytes + outstanding allocs) + Windows `GetProcessMemoryInfo` WorkingSetSize

## Configuration
- Cycles (objective → graph → result → machine verification → drop): 500
- Warm-up cycles: 10
- RSS tolerance: 8388608 bytes (8 MiB)

## Baseline (after warm-up)
- Live bytes: 69808
- Outstanding allocations: 24
- Working set: 3911680 bytes

## Final (after all drops)
- Live bytes: 69808 (delta +0)
- Outstanding allocations: 24 (delta +0)
- Working set: 3911680 bytes (delta +0)

## Drift observed during run
- Worst live-byte drift at a checkpoint: +0
- Worst alloc-count drift at a checkpoint: +0

## Verdict
- Allocator leak (bytes or allocs not back to baseline): **no leak**
- RSS within tolerance: **yes**

## Checkpoint log
```
cycle | live_bytes (delta) | live_allocs (delta) | RSS bytes (delta)
   50 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  100 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  150 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  200 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  250 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  300 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  350 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  400 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  450 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
  500 |        69808 (   +0) |          24 (  +0) |      3911680 (      +0)
```
