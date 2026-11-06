# What the loop claims—and what the repository actually shows

This map separates design intent, inspectable implementation, tests, historical execution, and measured comparative performance. A test function being present is not a fresh passing result. The November publication preparation did not perform a new Rust test run.

| Question | Inspectable source or test | What this supports | What it does not support |
|---|---|---|---|
| Can repair avoid stopping a proven-independent consumer? | [owner_return.rs](../crates/dc-domain/tests/cases/owner_return.rs): `returned_publication_blocks_its_consumer_without_stopping_proven_independent_consumer` | A precise domain-level scenario and assertions for selective blocking and preserved records | Real scheduling, a durable commit, or measured cost savings |
| Can old context be rejected after a dependency changes? | [context.rs](../crates/dc-domain/src/context.rs): `changed_dependency_basis_rejected` | Validation logic and a negative fixture for stale context basis | Optimal retrieval or complete enforcement in every live model request |
| Can continuation preserve the old attempt and obey a late stop? | [continuation.rs](../crates/dc-domain/tests/cases/continuation.rs): `continued_attempt_can_start_and_still_obeys_late_stop` | A versioned continuation proposal with fresh-basis and late-stop checks | Autonomous recovery of the complete desktop runtime |
| Is an unknown outcome different from confirmed failure? | [continuation.rs](../crates/dc-domain/tests/cases/continuation.rs): `unknown_outcome_at_any_scope_prevents_preparing_retry`; [recovery_evidence.rs](../crates/dc-watchdog/tests/recovery_evidence.rs) | Explicit uncertainty and reconciliation cases at component boundaries | Prevention of blind retries across the real broker route; the audit identifies a gap there |
| Can unrelated revisions leave an independent result admissible? | [work_contracts.rs](../crates/dc-domain/tests/work_contracts.rs): `unrelated_revision_allows_independent_result_but_read_change_does_not` | A testable distinction between unrelated revision changes and changed read dependencies | A complete incremental execution engine |
| Does a `Passed` label establish sound verification? | [October audit N03](../independent-review-20261001/02_FINDINGS.md) and [its probes](../independent-review-20261001/README.md) | A historical adversarial reproduction showing why the label alone is insufficient | A sound end-to-end verifier or a fixed N03 defect |

## Fresh publication checks

[VALIDATION.md](VALIDATION.md) and [VALIDATION.json](VALIDATION.json) contain the November preparation results. TypeScript checking and UI asset building passed. UI tests before and after localization each had 50 passing and the same three failing tests. The architecture checker failed with dependency-contract drift. These are packaging and limited build results, not an efficiency benchmark.

The October review separately records selected Rust tests and adversarial probes. Those results belong to their historical source and environment. A successful defect probe means the defect was reproduced, not that the application passed acceptance.

## Reproducing component tests

On a correctly configured Rust/MSVC environment, from the repository root:

```powershell
cargo test --locked -p dc-domain --test lifecycle_contracts returned_publication_blocks_its_consumer_without_stopping_proven_independent_consumer
cargo test --locked -p dc-domain --lib changed_dependency_basis_rejected
cargo test --locked -p dc-domain --test lifecycle_contracts continued_attempt_can_start_and_still_obeys_late_stop
cargo test --locked -p dc-domain --test lifecycle_contracts unknown_outcome_at_any_scope_prevents_preparing_retry
cargo test --locked -p dc-domain --test work_contracts unrelated_revision_allows_independent_result_but_read_change_does_not
```

These are reproduction instructions, not reported executions. Keep the full output, exact revision, environment, and exit code. Confirm that a positive number of matching tests ran: a filter matching zero tests is not evidence. Workspace resolution may still require dependencies for other packages. Use synthetic data and do not blindly run live-provider tests.

## Performance claims deliberately not made

This snapshot does not report an improvement in task success rate, tokens, latency, monetary cost, or autonomous self-development. There is no comparative dataset or statistical result establishing such an improvement. [EVALUATION_PROTOCOL.md](EVALUATION_PROTOCOL.md) defines what evidence would be needed.
