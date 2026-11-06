# Workgraph, worktrees and secure concurrency

## Workgraph instead of TODO list

Each work package is a node DAG with fields:

* objective;
* prerequisites;
* base SHA;
* authority keys;
* allowed paths;
* contracts consumed/provided;
* acceptance tests;
*negative tests;
* model class;
* time/token budget;
*evidence requirements;
* reviewer policy.

Node starts only when prerequisites have accepted receipts.

## Git worktree per writer

Example:

`git worktree add ../wt-WP-042 -b agent/WP-042 <base_sha>`

Each worktree gets a namespace:

* `PORT_BASE=42xxx`
* `DB_URL=...wp_042`
* `TMPDIR=.../wp-042`
* `OTEL_SERVICE_NAME=wp-042`
* separate browser profile/cache/logs.

This gives the agent the ability to actually run the application and not break its neighbors.

## File ownership is insufficient

Two agents can change different files, but one semantic contract. Therefore, in addition to allowed paths, you need `authority_key` lock.

Example: backend changes schema definition, frontend generates client types. Both affect `api:user:v3`; You cannot change the contract at the same time without a coordinated parent task.

## Conflict classes

1. **Textual** - git conflict. Easy to spot.
2. **Structural** - compile/type failure. CI is detected.
3. **Semantic** - both diffs compile, but the assumptions diverge. The most dangerous.
4. **Temporal/migration** - the order of application matters.
5. **Resource** - common port/db/cache/runtime.

Factory is required to have detector/routing for all five.

## Integration queue

Writer does not have the right to declare a product finished. It publishes candidate commit + receipt. Integrator checks stale-base risk, contract compatibility, rebases onto current candidate, launches scoped integration suite and only then changes candidate head.

##Replanning

If the reviewer detects a missing prerequisite, a new node is created, the original one is blocked by the dependency, and not endlessly re-prompted “try again”.
