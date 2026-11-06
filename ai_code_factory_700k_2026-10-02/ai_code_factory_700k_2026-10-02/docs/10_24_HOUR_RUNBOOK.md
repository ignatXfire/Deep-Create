#24-hour runbook

This is a plan for **already known product/requirements**. If PRD is not defined, the first hours will be spent on discovery and output will drop sharply.

## 00:00–00:45 — Constitution

Sol High + 3–6 architecture agents:

* mission/non-goals;
* domain map;
* dependency/layer rules;
* data/API contracts;
* quality gates;
* threat/reliability model;
* repository skeleton.

No mass writing yet.

## 00:45–01:30 — Workgraph

Qwen/Kimi + scouts build DAG, authority keys, acceptance tests, critical path. DeepSeek reviewers look for missing dependencies/duplicate packages.

## 01:30–02:15 — Harness bootstrap

Create:

* worktree launcher;
* per-task env namespace;
* CI shard runner;
*architecture lints;
* receipt schema;
* log/trace access;
* integration queue.

## 02:15–03:30 — Calibration wave

8–16 writers + matching reviewers/test lanes. The goal is not maximum LOC, but to measure throughput and find bottlenecks.

## 03:30 — Scale decision

If acceptance >70%, conflicts <5%, queues stable → 24–32 writers. Otherwise, fix contracts/harness.

## 03:30–07:00 — Wave 1

Core vertical slices. E2E starts immediately; don’t wait “for the backend/frontend to finish completely.”

## 07:00–12:00 – Wave 2 / scale

32–64 writers, 24–48 reviewers/tests. Critical path nodes are received by the best models. Generated surfaces are scaled separately.

## 10:00–14:00 — Continuous integration pressure

Some writers translate into integration defects, migration conflicts, observability, fixtures and test speedups.

## 12:00–16:00 — Feature convergence

Do not open large new epic nodes after cutoff. Complete vertical journeys.

## 16:00–19:00 — Reliability/security/performance

Crash/restart, stale state, authz, malformed inputs, cancellation, timeouts, resource leaks, load smoke, install/upgrade.

## 19:00–21:00 — Adversarial review

Kimi/DeepSeek cross-model reviewers + Sol High for highest-blast-radius diffs. Sol XHigh only on unresolved architectural/security decisions.

## 21:00 — Feature freeze

Candidate lock. Only verified fixes.

## 21:00–23:00 — Full gates

Fresh build, full tests, E2E matrix, package/install, recovery, security, artifact hashes, docs/checklists.

## 23:00–24:00 — Release judgment

No coding frenzy. Read exact release evidence, classify open risks, produce artifact + rollback plan.

## Key principle of the day

Don't put off checking until the last hour. Test/review/integration should flow next to generation from the first wave; otherwise by 20:00 there will be 700k WIP lines that are physically impossible to prove before midnight.
