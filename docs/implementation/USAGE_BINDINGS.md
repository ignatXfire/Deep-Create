# Relationship between the result and the expense already taken into account

Date: 2026-09-11. Section C-01 tested on artificial data. Reason - section `commit_verified_result` of document 16: the expense incurred does not depend on the acceptance of the contents and is not written off again.

## What error are we eliminating?

A successful result cannot be associated with the expense of another attempt or with an old entry for which a corrected invoice has already been received. A rejected answer cannot be considered free. An unknown remainder cannot be converted to zero when text verification is completed.

To achieve this, two related functions have been added. They do not create a new accounting system: they use the existing `LedgerPosition`, `apply_observation` and the rules for checking the result.

## Data path

1. The observation of the supplier or intermediary enters the already existing `apply_observation`. In the future, its authenticity is established by the corresponding trusted layer. This function stores the fact, including late billing or overrun.
2. `bind_usage` receives the expected list of attempt expense lines and the lines themselves. For each, the operation/restriction/window key, reserve number, resource type (including currency), version and attempt number are checked. The lists must correspond to each other without gaps, extra lines or repeated keys; The order of the input lines does not matter.
3. Closed `UsageBinding` is returned: the original reserve request, the exact expected line, status, contribution to accounting, excess sign and numbers of **already applied** observations. The numbers are taken from the string itself, not from the model's response.
4. `plan_accounted_result_transition` connects this binding to a proven plan for accepting or rejecting the result and its work unit. The number of the making attempt must match. The usual result check and all its prohibitions are still carried out.
5. The future C-03 should compare the versions of these lines along with the rest of the bases and record the associated changes in one indivisible operation. The account, eligibility, or status of the job may have changed between preparation and recording. The prepared plan does not change this.

## Different states cannot be mixed

| Observable state | What does binding save | What happens to the result |
|---|---|---|
| Consumption is finally known | Confirmed amount, version, observations | Acceptance is possible only after other checks |
| The part is known and the amount remains unknown | Confirmed part and the entire remaining limit, without release | This in itself does not mean the outcome of the action is unknown; existing restrictions must still be checked by the control layer |
| Proven non-sending | Recorded proof and already released state | The function does not re-release anything |
| Late bill exceeded reserve | Full consumption and sign of excess | Acceptance prohibited; deviation preserves the fact of excess |
| No sightings | No confirmation of consumption yet | Binding not created |
| The outcome of the action affecting the conclusion is unknown | This is a separate reason for blocking | Acceptance is blocked by existing `UnknownOutcome`, even with correct accounting linking |

`bind_usage` does not verify that all restrictions are current, the source is valid, or the validity of the action is valid. The unknown amount is retained, but it is **not permission** to run new queries or accept the result if the constraints are violated.

## Why new parts of code are needed

- `LedgerPosition::observation_ids` - reading the origin of already applied facts; does not create new facts.
- `ExpectedUsagePosition` is the exact expected string for future comparison before writing.
- `UsagePositionBinding` - immutable connection between the original request, version, state and evidence. The original request is saved in its entirety so as not to lose the intent and original limit.
- `UsageBinding` - closed set for one attempt. It cannot be parsed from JSON as a ready-made proof; this is verified by a compilation negative test.
- `AccountedResultPlan` - a single carrier of the result/unit plan and the read expense bases. It does not contain a recommission or release command.

Files: `crates/dc-domain/src/usage_binding.rs`, additions to `resource.rs` and `work/commit.rs`. The immediate callers now are tests; the future caller is the control layer and C-03. There are no networks, files, clocks, databases or new dependencies in these functions.

## What's checked

In `usage_bindings.rs` there are eight independent checks: re-preparation without accounting change, origin, unknown balance, empty/unobserved consumption, repetitions and rearrangements of lines, substitution of six parts of the base and attempts, late corrected bill and excess, proven zero consumption, size limits.

In `work_contracts.rs` - connection of acceptance and rejection with flow; prohibitions of someone else's attempt, incorrect evidence and inappropriate stage; the difference between an unknown amount and an unknown outcome; prohibition of acceptance if exceeded, with preservation of this fact if rejected. All 50 checks of this file and eight of the binding checks were run separately; full passage is specified in [STATUS.md](STATUS.md).

Kimi has prepared the groundwork. When enabled, added explicit checking for repetitions of input strings, searching by key without re-traversing the entire list, limiting the number of observations, and saving the full original query. DeepSeek looked at an intermediate option. The main agent then clarified the excess handling: it prohibits accepting the content, rather than storing the fact that has already happened; this difference is confirmed by a separate test. External opinion is not a substitute for these checks.

## Why isn't the obligation rewritten here?

`derive_completion_candidate` already infers execution from matching validated results. Rewriting `Obligation.status` and increasing its version on acceptance is not a harmless mark: the snapshot of the result contains the previous version of this line and will immediately become obsolete. This is reproduced by test `completion_uses_verified_findings_without_rewriting_obligation_basis`.

Therefore, the original definitions and historical snapshots are not modified. The individual stored performance evaluation and its relationships must be consistent with the present repository; You can't secretly correct a photo after the fact. The proposal of an external model to defer any such evaluations until acceptance by the owner is not accepted: internally verified results can be used by other works and earlier, under the exact conditions of communication.

## What remains

An empty list is not accepted as proof of the absence of consumption: there is still no clear path for truly local work without observations. Also not implemented is obtaining a complete list from the trusted account holder, up-to-date verification of rights/restrictions, secure storage of materials and the actual overall transaction. It is impossible to transmit an incomplete list, receive a blank plan and declare that the completeness of the present attempt has been proven.

When changing accounting, you need to jointly check: the meaning of the key and currency, the rules for re-observation, saving the unknown balance, the sign of excess, the exact versions of the binding and comparison before recording the result. Repeating a command with a lost receipt and turning off the power with this slice have not been tested.
