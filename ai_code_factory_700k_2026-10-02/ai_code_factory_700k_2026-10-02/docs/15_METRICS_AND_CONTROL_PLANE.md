# Metrics and adaptive control

## Golden metrics

### Flow
* ready nodes
* active nodes by role/model
* cycle time p50/p95
* queue age p50/p95
* packages accepted/hour

### Quality
* first-pass review acceptance
* integration pass rate
* E2E pass rate
* escaped defect/revert rate
* hallucinated path/symbol claims
* skipped-test anomalies

### Resources
* tokens/min by provider/model/role
* RPM/TPM headroom
* build CPU/RAM/disk utilization
* CI queue wait
* cache hit rate

### Change surface
* files changed/package
* authority keys/package
* LOC generated/accepted/integrated
* semantic-weighted accepted units

## Useful control ratios

`review_pressure = review_queue_age / writer_cycle_time`

`integration_pressure = integration_queue_age / writer_cycle_time`

`waste = 1 - integrated_changes / produced_changes`

`proof_cost = verification_wall_time / accepted_package`

## Auto-control examples

* If `review_pressure > 1.5`, stop dispatching lowest-priority writers and spawn/reassign reviewers.
* If `integration_pressure > 2`, freeze noncritical features.
* If same authority key causes >3 conflicts/hour, serialize that subtree or redesign boundary.
* If a model's path-hallucination rate doubles, demote it from reviewer to implementation until re-evaluated.
* If one provider reaches >85% token bucket for sustained interval, reserve capacity for repair/review and route new work elsewhere.

## Dashboard

Dashboard should show **candidate SHA at top**. Every green light is linked to receipt and artifact. Historical greens are visually separate.

Recommended panels:

1. Critical path DAG.
2. Queue/bottleneck heatmap.
3. Model acceptance/latency matrix.
4. Build/E2E farm capacity.
5. Current candidate gate matrix.
6. Top blockers with owner/evidence.
7. WIP by authority key.

## No vanity metrics

Raw token usage, agent count and LOC are secondary. A factory can burn billions of tokens and still produce zero release-verified value.
