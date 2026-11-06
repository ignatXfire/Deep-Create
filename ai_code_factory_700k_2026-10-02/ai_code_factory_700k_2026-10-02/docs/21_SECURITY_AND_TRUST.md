# Security, trust and blast-radius control

## Agent permissions are role-based

* Scout/planner: read-only repo/web/docs.
* Writer: scoped worktree mutation, local commands, no production credentials.
* Reviewer: read + test, normally no candidate write.
* Integrator: candidate mutation, no production deployment secret unless separately authorized.
* Release automation: artifact signing/deployment under deterministic policy.

## Secrets

Never place secrets in prompts/transcripts. Use short-lived credential injection at process boundary, allowlisted endpoints and redaction. Store only secret identifiers in receipts.

## Supply chain

Dependency additions are explicit work items: license, maintainer/reputation, version pin, vulnerability scan, transitive impact. High-throughput agents must not casually add packages to fix tiny problems.

## Command policy

Sandbox/worktree by default. Destructive commands, external side effects, financial actions, production changes and irreversible migrations require higher authority.

## Code provenance

Every accepted commit links to:

* task;
* run/attempt;
* model/provider;
* base/head SHA;
* evidence receipt;
* reviewer;
* integration gate.

This makes a 700k-LOC day auditable.

## Adversarial agents

Dedicated red-team lanes should search for authz bypass, injection, insecure deserialization, path traversal, secret leaks, race conditions, dependency confusion and unsafe process execution. Their role is not to approve; they produce findings/tests.

## Release trust

Final artifact hash must correspond to tested release SHA and build recipe. A passing source tree plus a different binary is not acceptable evidence.
