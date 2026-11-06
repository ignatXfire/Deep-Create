# Failure modes of a large multi-agent factory

## 1. Agent avalanche

Symptom: dispatch rate is high, integration queue is growing, useful output stops increasing.

Fix: backpressure; workers are translated to tests/review/integration.

## 2. Duplicate implementation

Two agents independently decide on one hidden prerequisite.

Fix: global workgraph + authority lock + duplicate detector by touched contracts/symbol graph.

## 3. Contract drift

Each module is locally green, globally the interfaces are incompatible.

Fix: versioned schemas, generated bindings, consumer/provider tests, centralized architecture lint.

## 4. Reviewer theater

Reviewer writes a compelling PASS without reproducible verification.

Fix: exact SHA, file/symbol existence checks, command receipts, fresh adversarial test, machine verifier.

## 5. Stale-base green

Worker was testing a commit from an old candidate, and the merge train had already changed the dependency.

Fix: integration rebase + rerun affected gates after rebase.

## 6. Test illusion

Suite reports `0 failed`, but the required test was ignored/skipped/not discovered.

Fix: expected discovered-test count, forbid unexplained skipped/ignored, preserve raw logs.

## 7. Shared environment poisoning

Agents use the same DB/port/browser/cache and influence each other.

Fix: worktree/container namespaces and disposable environments.

## 8. Context landfill

A huge master prompt clogs up the context and quickly becomes outdated.

Fix: short map + progressive docs/context retrieval.

## 9. Generated slop propagation

A bad pattern gets into the repo, other models copy it hundreds of times.

Fix: golden principles as mechanical lints + recurring garbage-collection/refactor agents.

## 10. High LOC / low value

Agents are optimized for files and strings.

Fix: score accepted user journeys/contracts/semantic units, never reward raw LOC alone.

## 11. Quota starvation

Writers eat up provider token budget, reviewers cannot complete gates.

Fix: per-role reserved token buckets and priority queues.

## 12. Integration serial bottleneck

One candidate gate is too slow.

Fix: hierarchical integration: module candidates -> domain candidates -> product candidate; expensive full gates run continuously, not after every line.

## 13. Premature feature freeze or endless feature creep

Fix: explicit cutoff time, critical-path budget, release-class priority.

## 14. False autonomy

Swarm waits for a person for each blocker or, conversely, changes global contracts without authority.

Fix: reversible/autonomous within scoped authority; contract-breaking work becomes explicit parent decision node.
