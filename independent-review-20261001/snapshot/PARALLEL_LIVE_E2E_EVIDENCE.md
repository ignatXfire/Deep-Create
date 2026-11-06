# PARALLEL_LIVE_E2E_EVIDENCE.md

**Date:** 2026-10-01T05:30:09.026328800+00:00
**Endpoint:** http://localhost:20128
**Model:** dash/kimi-k3
**Graph:** diamond DAG (A -> B, A -> C, B+C -> D)

## Result
```
parallel execution: 3 levels, 4 units, max parallelism 2, 4 success, 0 failed, 4 verified, 0 verification_failed | timings=[{"unit_id":"00000000000070008000000000000064","level":0,"start_ms":12,"duration_ms":173392,"ok":true},{"unit_id":"0000000000007000800000000000012c","level":1,"start_ms":173414,"duration_ms":60975,"ok":true},{"unit_id":"000000000000700080000000000000c8","level":1,"start_ms":173414,"duration_ms":61213,"ok":true},{"unit_id":"00000000000070008000000000000190","level":2,"start_ms":234628,"duration_ms":62228,"ok":true}]
```

## Wall clock
296918 ms

## Timings
```json
[
  {
    "duration_ms": 173392,
    "level": 0,
    "ok": true,
    "start_ms": 12,
    "unit_id": "00000000000070008000000000000064"
  },
  {
    "duration_ms": 60975,
    "level": 1,
    "ok": true,
    "start_ms": 173414,
    "unit_id": "0000000000007000800000000000012c"
  },
  {
    "duration_ms": 61213,
    "level": 1,
    "ok": true,
    "start_ms": 173414,
    "unit_id": "000000000000700080000000000000c8"
  },
  {
    "duration_ms": 62228,
    "level": 2,
    "ok": true,
    "start_ms": 234628,
    "unit_id": "00000000000070008000000000000190"
  }
]
```

## Parallelism proof
- B start=173414 ms, duration=61213 ms
- C start=173414 ms, duration=60975 ms
- Overlap: true

## Unit stages
```
00000000000070008000000000000064 -> Accepted
000000000000700080000000000000c8 -> Accepted
0000000000007000800000000000012c -> Accepted
00000000000070008000000000000190 -> Accepted
```
