# C-01: actually implemented work section

Updated: 2026-09-11. All entrances and tests are artificial. This is a pure core of rules, not a working implementer and not a full implementation of documents 14–16/24.

## Files and calling boundary

- `crates/dc-domain/src/work.rs`: purpose, obligations, main work, unit, structural limits and verification of the agreed graph affected.
- `crates/dc-domain/src/work/links.rs`: precise inter-worker connections, catalog of provided results and output of conditions/lines read.
- `crates/dc-domain/src/work/result.rs`: Dependency snapshots, valid package transitions, completion candidate, and separate owner acceptance reason check.
- `crates/dc-domain/src/work/lifecycle.rs`: Limited unit/attempt/root transition plans with precise expected strings and private events.
- `crates/dc-domain/src/work/continuation.rs`: limited clean plan for a new attempt after a confirmed failure; boundaries - [CONTINUATION_NEW_ATTEMPT.md](CONTINUATION_NEW_ATTEMPT.md).
- `crates/dc-domain/src/work/commit.rs`: a single plan for replacing the result and its unit, with an unchangeable attempt as a reading condition. The exact boundaries are [RESULT_TRANSITIONS.md](RESULT_TRANSITIONS.md).
- `crates/dc-domain/tests/work_contracts.rs`: integration clean checks.
- `crates/dc-domain/tests/lifecycle_contracts.rs`: Individual lifecycle checks.

These functions do not read or change any files, network, clock, environment, secrets or database. Their immediate executable caller today is tests. Future C-05/C-08 should convey a consistent canonical view, and C-03 should check for expected strings and commit the decision atomically. The success of a pure function is not a commit receipt, entitlement or reservation.

## Public functions

| API | Factual Check |
|---|---|
| `validate_objective` | Non-empty bounded target, owner source, exact criteria, commitment membership, no parent cycles, explicit reason for abandonment of commitment, preservation of passed owner constraints |
| `validate_work_graph` | The exact version of the affected graph; identities and domains; coverage of non-cancelled machine verifiable obligations; final steps/attempts/extensions; internal DAG; precise sources/consumers of interworking connections; expected outputs and future cycles, including owner acceptance node |
| `assess_link` | `Pinned`, `FollowHead`, `AwaitOutput` exact edition; `True/False/Unknown`; read conditions even with `False`; mandatory state of admissibility of fixed provision; separate exact acceptance by owner |
| `graph_dependency_rows` | Canonical strings owned by the passed graph; policies and settings are transferred separately by their owner |
| `validate_dependency_snapshot` | Accurate limited read-set and policies, no duplicates, versions and fingerprints provided. No check via general `EventSeq` |
| `transition_result` | Expected versions of mutable rows, active attempt, target/unit/snapshot revisions, required communication grounds, exact compliance of check to package/fingerprint/policy/criteria; adding `RowVersion` by exactly one with closed overflow failure |
| `plan_result_transition` | Related package and unit replacements; exact match of the unit with the graph, area, attempt, launch generation and versions; try only `Submitted`; blocking installation, main operation and unit; binding the accepted result only from the returned packet |
| `plan_accounted_result_transition` | Acceptance/rejection is related to the already observed cost of this attempt; there is no re-write-off; exceeding prohibits acceptance, but is not erased upon rejection. See `USAGE_BINDINGS.md` |
| `derive_completion_candidate` | Coverage of obligations with accepted packages with accurate checks and still valid dependencies; spaces instead of false completion; owner's deferred judgment separately |
| `validate_owner_acceptance` | Matching of the exact candidate, edition of the goal, reviewed results and owner's obligations; separate positive outcome of the final inspection. Event Proxy Sets External Authenticated Path |
| `plan_unit_attempt_transition` | Prepare/start/observe submit/cancel/lose one attempt; exact scope/unit/attempt/run-generation and versions; opaque plan for related replacements and events, or only maintaining late observation |
| `plan_continuation_new_attempt` | Transmits only the confirmed failed `Blocked` unit of the new attempt identity in the pair `Leased + Prepared`; checks the full set of gates, immutable bases and a hard retry counter; does not resume, dispatch or record |
| `validate_final_review`, `plan_root_transition` | Path draft → executing → completion_candidate → verifying → awaiting_owner_acceptance → owner_accepted; opaque candidate/check/owner decision; re-derivation of the candidate according to the current agreed upon form before each final transition |
| `plan_owner_return` | Returning the exact candidate after the accepted reason analysis; preservation of restrictions and history, verification of repair requirements, associated change in the admissibility of grants. See `OWNER_RETURN.md` |
| `plan_owner_rejection` | Acceptance of exact refusal before parsing: pending entry, targeted prohibition of old acceptance and own time limits on grants; without overwriting old reasons. See `OWNER_REJECTION.md` |

The semantic stages `RootStage`/`UnitStage` are not mixed with the set `BlockingReason`. Clearing one element of a set does not clear the rest. Lifecycle does not clear boundaries or issue credentials. The lock prevents prepare/start; maintaining an observation after stopping does not advance the unit into result or acceptance. Cancellation and process loss recording remain valid in their exact stage.

`ResultPacket` only supports proven transitions `Submitted → Verifying → Accepted|Rejected`, as well as `Submitted|Verifying → Stale` when actually changing the base. An already accepted package is not retroactively rewritten into an obsolete one. The function returns a new entry and does not change the input on failure.

After an independent source-review, two real gaps are closed: accepting the result requires the corresponding unit stage, and the internal predecessor requires the canonical accepted-result binding and the exact `UnitResultBinding`/`AcceptedResult` snapshot lines. `Cancelled`, `Cancelling`, `Lost` and active blocking of the main work do not produce semantic `Accepted`. `MustAuthorize` is closedly rejected in this slice: the accepted result of another unit is not itself a right.

## Reproduction and verification completed

From the project root in PowerShell:

```powershell
. .\tools\Enter-BuildEnvironment.ps1
cargo test -p dc-domain --test work_contracts
cargo test -p dc-domain --test lifecycle_contracts
```

As of 2026-09-11, **50 graph/result checks were separately performed - all successful**. The number and outcome of the last total pass, including life cycle, is given in [STATUS.md](STATUS.md) and the associated machine report. The compilation used the project's prepared environment. Verified:

- foreign internal dependency area, foreign obligation/consumer, no coverage, unlimited unit, old version of the affected graph;
- a cycle of still future conditional outputs with the current `False`, a cycle through waiting for acceptance by the owner, allowed historical mutual references and the final sequence A1 → B1 → A2;
- `AwaitOutput` edition 2 does not accept the output of edition 3; the root stage `OwnerAccepted` does not itself replace the exact record of accepting the desired output;
- the fixed result is not replaced by a new head; its changed validity still invalidates the snapshot; `FollowHead` reads its own binding;
- change `False → True`, unknown condition, unknown field, missed interworking connection and missed `PublicationState` do not give a positive transition;
- you cannot skip a check, accept another fingerprint/snapshot, accept a self-check of the same attempt, apply a later revision of a unit/attempt/line;
- changing an independent commitment does not block another outcome; changing an obligation that has actually been read blocks it;
- the completion candidate does not accept the work by the owner; revocation of the used basis prohibits the new candidate, preserving the old accepted package;
- version overflow; unknown fields and options for wire forms; zero and number outside the limit `u32`; 64 deterministic sequences of 20 transition attempts with input saving in case of failure.
- the mandatory internal predecessor cannot be circumvented by the absence of a snapshot line; another accepted-result binding makes the consumer obsolete; canceled/lost unit and root gate do not accept the result.
- Each unit has a commitment commitment, even if other units already cover the entire goal.
- Empty versions of commands, conditions and blocking reasons do not accept extra fields; duplicate keys and unknown variants are rejected.
- Candidate A cannot be mixed with final test B; the area and stage of each base are consistent. The acceptance event stores the exact decision number of the owner, candidate, and final review.
- The future launch number is rejected; Late observation of the old launch does not provide meaningful progress. Impossible unit/attempt stage pairs and the accepted result of the unaccepted unit are rejected.

## Borders that cannot be hidden

1. Completeness of the agreed upon affected graph is the obligation of the caller C-03. A pure function cannot prove that a conflicting node is not missing from the database. The graph has an expected version, but there is no transactional check.
2. `ResultPublication`, `VerificationRecord`, owner records and validity states are accepted as canonical records of the authenticated layer. This code does not create a publication, verify archive bytes/checkpoints, verify a signature, or qualify verifier independence. Another `attempt_id` is only a minimal apparent self-test failure, not a C-08 proof of independence.
3. The freshness policy is version bound, but the time expiration is not calculated. Checks for current time, read/expand/action permissions, resources and fencing are performed separately before actual application.
4. `Informs` does not form a mandatory wait graph. In actual use, it must be included in `used_links` and read-set; clean code is unable to detect hidden material that the untrusted executor has not listed. Such provenance should install C-07 on the actual collected order.
5. `WorkUnitSpec` - intentionally partial structural form. Full capability/data-class/resource/exploration-budget/workspace contracts have not been replaced by stubs and have not yet been integrated here.
6. Lifecycle returns only pure replacement plans for associated rows and events; queue, saving events, common C-05 dispatcher, registering new attempts after failed/lost, checking the global uniqueness of attempt ID and no actual recovery. Preparation here does not reserve a budget and does not issue a lease. There are no changes to the overall workspace, durability, external actions, models, actual scheduler, or UI. Graph error `Cycle` does not yet contain the minimum outline for the interface.

## Next shortcut after lifecycle

Necessary further connections not yet implemented by this slice:

1. The associated result and unit plan and its connection to the observed flow rate is implemented in `commit.rs`; attempt and expense remain the exact reading conditions. Next is a trusted complete list of expenses, rights/restrictions before recording and real storage. Fulfillment of obligations is derived from verified results; Naive overwriting of the original commitment string makes the snapshot obsolete and is disallowed by the new test. Details - `USAGE_BINDINGS.md`.
2. Implement limited retries with a preserved history and checking the global uniqueness of the ID; do not reuse a canceled/lost attempt. Don't confuse your own lifecycle with proof of workflow.
3. `plan_owner_rejection` registers the failure before parsing and creates an accurate wait; `plan_owner_return` claims this entry, clears the current candidate, saves the history, and only releases its own wait/proven independent time constraint. Other causes and consumers are protected. Real authenticated input, semantic verification of the reason, explanation of the already fulfilled condition, change of goal and creation of a new attempt are still missing. Details - `OWNER_REJECTION.md` and `OWNER_RETURN.md`.
4. Change blocking only through specific specific reasons/events. A matrix of 14 causes and 11 classes of operations has already been implemented and is checked for all combinations of causes; Added targeted withdrawal for pending refusal. See `LIFECYCLE_GATES.md`. The trusted determination of the class of the present action and the completeness of the lock chain remain the responsibility of the future control layer.
5. C-03 must check and commit all expected lines and events in one transaction. Instruction idempotency, lost receipt, and durability are not proven by these clean tests.

These points are a plan for the next cut, not an approval of the completed check.
