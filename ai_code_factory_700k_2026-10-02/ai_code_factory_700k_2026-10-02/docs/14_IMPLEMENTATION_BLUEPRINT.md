# Implementation blueprint

## Minimal services

### `planner`
Consumes mission + repo map; emits versioned workgraph.

### `scheduler`
Token-bucket aware; chooses model/role; obeys DAG, authority locks, concurrency/backpressure.

### `workspace-manager`
Creates/deletes git worktrees or containers; assigns env namespaces; records base SHA.

### `agent-runner`
Runs CLI/provider agent; streams tool events; enforces wall time/tool policy; persists transcript metadata.

### `evidence-collector`
Captures command, exit code, tests, logs, hashes, environment fingerprints.

### `review-controller`
Assigns independent reviewer; validates review receipt against actual diff/files/symbols.

### `integrator`
Owns candidate branch; rebases; runs affected gates; publishes new candidate atomically.

### `e2e-farm`
Runs browser/desktop/API/user journey tests on isolated candidate builds.

### `metrics-controller`
Calculates throughput/queues/acceptance and changes concurrency.

## Storage

Use a transactional DB (SQLite is sufficient for one machine; Postgres when distributed) plus content-addressed artifact storage.

Tables/entities:

* missions
* work_packages
* dependencies
* authority_locks
* runs
* receipts
* reviews
* integration_attempts
* artifacts
* gate_results
* model_stats

## State transition example

Scheduler can move `SPEC_READY -> ADMITTED` only if prerequisites accepted + authority lock acquired + provider budget exists.

Agent completion moves to `LOCAL_GREEN` only if receipt validator confirms required commands/hashes. Agent text cannot mutate state directly.

## Reconciliation loop

Every few seconds:

1. refresh active runs;
2. mark crashed/timed-out runs;
3. verify receipts;
4. release expired locks;
5. unblock DAG nodes;
6. compute queues and quotas;
7. scale/reassign pools;
8. dispatch next highest-value ready nodes;
9. persist decision event.

## Failure recovery

Every transition idempotent. Every run has immutable attempt ID. Retry creates a new attempt linked to previous evidence; it never rewrites failed evidence.

## Deployment topology

Single workstation: orchestrator + many local worktrees + remote LLM APIs + local build cache.

High scale: orchestrator/Postgres + worker VMs/containers + shared content-addressed cache + CI runners + provider gateways.

## Critical design constraint

Do not let LLM prose be the control plane. LLM proposes actions/artifacts; deterministic code validates and commits state transitions.
