# Extended Stability Soak Test Evidence

- **Date**: 1790842997
- **Duration**: 1800 seconds (30.0 minutes)
- **Total cycles**: 1780000
- **Overall throughput**: 988.65 cycles/sec
- **Errors**: 0

## Memory (RSS) Time Series

| Elapsed (s) | RSS (KB) | RSS (MB) |
|-------------|----------|----------|
| 0 | 3944 | 3.85 |
| 60 | 7084 | 6.92 |
| 120 | 8804 | 8.60 |
| 180 | 10328 | 10.09 |
| 240 | 11732 | 11.46 |
| 300 | 13376 | 13.06 |
| 360 | 14848 | 14.50 |
| 420 | 15792 | 15.42 |
| 480 | 16560 | 16.17 |
| 540 | 17328 | 16.92 |
| 601 | 18048 | 17.62 |
| 661 | 18752 | 18.31 |
| 721 | 19444 | 18.99 |
| 781 | 20276 | 19.80 |
| 841 | 21236 | 20.74 |
| 901 | 22160 | 21.64 |
| 961 | 23184 | 22.64 |
| 1021 | 23888 | 23.33 |
| 1081 | 24288 | 23.72 |
| 1142 | 24992 | 24.41 |
| 1202 | 25888 | 25.28 |
| 1262 | 26720 | 26.09 |
| 1322 | 27572 | 26.93 |
| 1382 | 28384 | 27.72 |
| 1442 | 29024 | 28.34 |
| 1503 | 29556 | 28.86 |
| 1564 | 30068 | 29.36 |
| 1624 | 30580 | 29.86 |
| 1684 | 31028 | 30.30 |
| 1744 | 31668 | 30.93 |
| 1800 | 32436 | 31.68 |

## Throughput by 5-Minute Window

| Window | Cycles | Cycles/sec |
|--------|--------|------------|
| 0-5 min | 540661 | 1802.20 |
| 5-10 min | 298340 | 994.47 |
| 10-15 min | 261666 | 872.22 |
| 15-20 min | 258073 | 860.24 |
| 20-25 min | 236288 | 787.63 |
| 25-30 min | 184972 | 616.57 |

## Cycle Time Statistics

- Min: 0 ms
- Median: 0 ms
- P95: 0 ms
- P99: 3 ms
- Max: 208 ms
- First-quarter median: 0 ms
- Last-quarter median: 0 ms
- Slowdown ratio: 1.00x

## Memory Analysis

- Baseline RSS: 3944 KB
- Final RSS: 32436 KB
- Peak RSS: 32436 KB
- RSS growth: 28492 KB (29175808 bytes)

## Verdict

**PASS — System stable over 30-minute soak**
- Throughput stable: true (slowdown ratio 1.00x)
- RSS plateauing: true (growth 28492 KB)
- Errors: 0
