# Testing and verification factory

## Why tests should scale with your code

With dozens of writers, manual testing becomes the main bottleneck. Therefore, each implementation lane must have a parallel verification lane.

## Test pyramid for agent-first development

### Layer 0 - mechanical
Format, lint, typecheck, forbidden dependency edges, secrets, schema validity, generated-files freshness.

### Layer 1 - unit/property
Boundary conditions, state transitions, serialization, algebraic/property tests, fuzz where appropriate.

### Layer 2 - contract
Provider/API/DB/message contracts. Consumer/provider compatibility.

### Layer 3 - integration
Real database/cache/filesystem/process boundaries in isolated env.

### Layer 4 - product E2E
Critical user journeys from actual launch through persisted result.

### Layer 5 — reliability/security/performance
Crash/restart, timeouts, cancellation, retry/idempotency, injection, authz, resource exhaustion, load/perf budgets.

## RED/GREEN protocol for critical changes

For bug/security/state-machine work:

1. Run a new test on pinned base and save **real RED**.
2. Implement fix.
3. Get GREEN on candidate.
4. Launch surrounding regression suite.
5. Independent reviewer reproduces the key review.

A test that has never been red may not test anything.

## False-green defense

Receipt is obliged to store:

* exact command;
*cwd/container/worktree;
* base SHA/head SHA;
* discovered test count;
* passed/failed/skipped/ignored;
* raw process exit code;
* stdout/stderr artifact hash;
* dependency lock hash;
* environment fingerprint.

It is prohibited to turn `grep summary` or the agent's response into evidence of a successful process.

## Independent review

Reviewer:

* not the author of diff;
* gets exact base/head;
* first reads diff/contracts, then claims of the author;
* checks the existence of each cited file/symbol;
* selects at least one new adversarial test;
* can return `PASS`, `FAIL`, `BLOCKED`, `INSUFFICIENT_EVIDENCE`.

Model diversity is useful, but a single model is also hallucinatory. The source of truth is the repository and executable proofs.

## Mutation/adversarial generation

The reviewer pool part should intentionally break assumptions: remove required checks, reorder events, corrupt persisted state, kill process, simulate stale versions, malformed payloads, partial network failures.

## Test selection

Don't run the full suite on every small edit if it takes hours. Use affected-test map for local gate, and run full candidate suite continuously on integration branch and always before freeze/release.
