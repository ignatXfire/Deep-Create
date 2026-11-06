# CI / build farm for extreme throughput

## Why build infrastructure dominates at scale

64 writers can finish patches faster than one machine can compile/test them. A code factory therefore needs a build/test farm as seriously designed as the model router.

## Hierarchical gates

### Worker gate
Fast format/lint/type/unit/affected tests, target 1–5 minutes.

### Domain integration gate
Contract + domain integration, target 5–15 minutes.

### Candidate gate
Full workspace + E2E/security/install, continuous and parallel, may take longer.

## Caching

* language dependency cache pinned by lock hash;
* compiler incremental/remote cache where safe;
* Docker/build-layer cache;
* pre-warmed base images;
* fixture snapshots;
* avoid sharing mutable caches that can contaminate tests.

## Sharding

Shard by test ownership or historical duration, not arbitrary file count. Long-tail shard determines wall time.

## Priority

1. release/candidate failures;
2. critical-path work packages;
3. repair validation;
4. normal writer validation;
5. speculative/refactor jobs.

## Resource estimation

For 48 writers with median 20-min implementation and 4-min worker gate, steady state can request roughly 9.6 worker-gate executions/minute if synchronized badly. Smooth queueing and enough runners to keep p95 wait below ~20–30% of writer cycle.

## Observability

Collect compile/test duration by package, cache hit, flakes, runner failures, queue wait, CPU/RAM/disk/network. Scheduler uses these metrics to choose concurrency.

## Test determinism

Quarantine is dangerous if it simply hides failures. Track flaky tests explicitly, retry only with evidence, and prevent a newly flaky critical test from silently becoming ignored.
