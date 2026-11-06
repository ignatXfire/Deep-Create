# Associated change in result and work unit

Date: 2026-09-11. A limited section of C-01 has been implemented. This is not a confirmation of storage, execution, or entire system readiness.

## What problem does it solve?

Previously, `transition_result` checked the new result record and the caller separately had to change the work unit correctly. A connection error could leave a received packet without a corresponding unit binding. `plan_result_transition` now returns both matched replacements and the event in one closed plan. External code cannot independently assemble such a plan through its fields or parse it from JSON.

The old function was left as a general checking mechanism, rather than its checks being copied to another place. It is still needed to check the result separately. Returning it does not in itself allow the package to be saved separately from the associated changes.

## Data path and responsibility

1. The future control layer receives the authenticated rows: graph, batch, attempt, expected versions and the complete lock chain. The model text does not validate this data.
2. `ResultTransitionContext` groups the bases. Root locks are read from the graph; The caller passes additional set and unit locks. C-01 cannot prove that the caller did not hide the lock or row.
3. `plan_result_transition` requires a complete match of the transmitted unit with the graph unit, and not just the same number. Verifies area, unit, active attempt, run generation, and expected versions. Only allows an attempt that has already produced a result (`Submitted`).
4. `transition_result` checks the validity of the transition, the relevance of the read grounds and the exact proof of the check. Acceptance requires a proper way of testing, criteria and suitability of the method, not just the word "successful".
5. The plan receives a new unit, a new package and an event. A version overflow of either of the two entries aborts the preparation. The input data is not changed on either success or failure.
6. The future C-03 is required to compare the expected rows, attempt and dependencies again before a single record. Successful plan preparation does not replace this comparison: the state may change between read and write.

## State matching

| Result | Work unit | Additional condition |
|---|---|---|
| Submitted → Verified | Result presented → checked | The grounds are relevant; all three levels of locks allow transition |
| Reviewed → accepted | Reviewed → accepted | Accurate positive evidence; the binding of the new package is saved |
| Under review → rejected | Under review → rejected | Accurate non-positive evidence; no accepted binding |
| Submitted or reviewed → deprecated | Corresponding state → obsolete | The checking function did indeed detect the missing base; You cannot arbitrarily declare a current package obsolete |

Deviation and obsolescence preserve the fact; a protective stop in itself should not force you to lose such a fact. They do not accept the result and do not allow new work. A packet that has already been received is not retroactively rewritten in this way.

## Why return fields are needed

- `packet()` and `unit()` are related replacements. `accepted_result` appears only upon acceptance and contains the result number, snapshot, fingerprint and version of the new package.
- `expected_basis()` - area, unit, previous version of the unit, number and version of the attempt, generation of the launch. The attempt here is a read condition, **not a write to re-save**. Otherwise, it would be possible to overwrite the parallel cancellation with an unmodified old copy.
- `expected_result_row_version()` - package version before change.
- `event()` - proposed event: area, unit, result, attempt, previous and new states, number of the test used. An unverified record cannot be included in the transition to verification or obsolescence event under the guise of evidence.

These fields have no side effects. They do not grant the right, fulfill the request, or certify the durability of the record.

## What is checked

The seven initial stand-alone tests in `work_contracts.rs` test the full path to acceptance, no source modification, 14 base mismatches, all invalid try states, each of the three blocking levels, rejection and obsolescence, missing/incorrect/irrelevant evidence, and each version overflow. Later, checks for connection with expense and preservation of the basis of obligations were added. The current total of this file is: 50 successful tests; general report - in `STATUS.md`.

Kimi prepared the initial proposal. Prior to inclusion, the main agent replaced the non-existent call, added a separate unit lock chain, eliminated the unproven test reference in the event, reduced repetition of existing tests, and wrote tests independently of the proposal. After that, DeepSeek found no confirmed defect in the limited set of transferred functions. Such review is an additional opinion, not proof of the absence of errors.

## What cannot be lost with further changes

With a new result state, the permissible initial states, unit stage compliance, permissibility under locks, result binding, and event are simultaneously reviewed. When the version/generation changes, the conditions for reading and checking the re-command are revised. You cannot replace the check of the entire unit with a comparison of one number or consider the old attempt to be current after a new start.

`ResultTransitionPlan` itself remains the package and unit plan. Additional `AccountedResultPlan` already associates it with the observed consumption of the exact attempt without re-writing; [details](USAGE_BINDINGS.md). A complete contract 16 also requires certified origin, valid rights and restrictions, completeness of the grounds read, and a single entry. Compliance is now inferred from the verified results, and the original lines and snapshots are not rewritten as a completion mark. It is impossible to keep one pure plan and call it complete acceptance. The base, lost receipt, and power outage checks for this path have not yet been completed.
