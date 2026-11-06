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
- Working set: 3887104 bytes

## Final (after all drops)
- Live bytes: 69808 (delta +0)
- Outstanding allocations: 24 (delta +0)
- Working set: 3895296 bytes (delta +8192)

## Drift observed during run
- Worst live-byte drift at a checkpoint: +0
- Worst alloc-count drift at a checkpoint: +0

## Verdict
- Allocator leak (bytes or allocs not back to baseline): **no leak**
- RSS within tolerance: **yes**

## Checkpoint log
```
cycle | live_bytes (delta) | live_allocs (delta) | RSS bytes (delta)
   50 |        69808 (   +0) |          24 (  +0) |      3887104 (      +0)
  100 |        69808 (   +0) |          24 (  +0) |      3891200 (   +4096)
  150 |        69808 (   +0) |          24 (  +0) |      3891200 (   +4096)
  200 |        69808 (   +0) |          24 (  +0) |      3891200 (   +4096)
  250 |        69808 (   +0) |          24 (  +0) |      3891200 (   +4096)
  300 |        69808 (   +0) |          24 (  +0) |      3895296 (   +8192)
  350 |        69808 (   +0) |          24 (  +0) |      3895296 (   +8192)
  400 |        69808 (   +0) |          24 (  +0) |      3895296 (   +8192)
  450 |        69808 (   +0) |          24 (  +0) |      3895296 (   +8192)
  500 |        69808 (   +0) |          24 (  +0) |      3895296 (   +8192)
```
