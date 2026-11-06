# Acceptance of owner’s refusal before analyzing the reason

Date: 2026-09-11. Pure contract `14.OwnerRejectionPlan/v1`, connected to `14.OwnerReturnPlan/v2`. This is the implementation of the rules in Rust, the window button is not connected and the owner's authenticated channel is not working.

## Which gap is closed

Previously, there was only a return after an already accepted analysis of the reason. Between the receipt of the refusal and such analysis there was no separate executable protection yet.

`plan_owner_rejection` now accepts an exact trusted refusal without waiting for parsing. In the proposed state, the candidate and final check are retained, but a pending failure record and address ban `OwnerRejectionPending { owner_event_id }` appear. The previous confirmation can no longer accept this result. The semantic stage remains awaiting acceptance: further work is not yet permitted.

For the provision of this work and the current goal, the impact is unknown at this time. Their actual permissibility becomes unresolved; old unresolved and revoked states are preserved. The original results and evidence are not rewritten or declared false.

## Subsequence

| Moment | Main job | Provided result | What is allowed by this rule |
|---|---|---|---|
| To failure | Awaiting acceptance of the exact candidate | Valid | Routine individual clearance checks |
| Failure registered | Same stage, but there is a precise pending entry and a ban | Unknown Impact: Unresolved | Get the fact, analyze the reason, prepare a plan; not accepting the outcome and not fulfilling the consequence |
| Dismantling requires repair | Return to execution, old valid candidate removed | The affected grounds remain unresolved | Remove only your own refusal ban; further execution goes through the remaining checks again |
| Separate output is proven independent | As part of the same return plan | You can only remove your own unchanged time limit | New version of admissibility; Previously issued images are not overwritten |

The first and third transitions are separate **entire sentences of the entry**. Until C-03 is actually confirmed, none of them can be declared accomplished. The future writer should resolve the unknown outcome of the recording and repeat the command according to the saved history, and not by rewriting or creating a new identity.

## What is stored and why

- `PendingOwnerRejection`: the exact owner event, reason, scope, return candidate with final check, and only those time constraints that this failure created.
- `PublicationRejectionHold`: pair of states “before limit → established state”. It is closed from arbitrary construction by the calling code and JSON parsing. Matching a textual reason alone is not enough to own a constraint.
- `OwnerRejectionPlan`: work state replacement, all read grant states, required replacements and expected versions of the main work, target and graph.
- `RootLifecycleState.pending_owner_rejection`: Active pending entry. The overall condition check compares it to the region, the candidate, the final check, and the presence of the exact cause in the prohibition set.
- `OwnerReturnPlan.resolved_rejection()`: persisted wait release basis, including initial pairs of temporary states; Clearing a valid field does not mean deleting the history.

The `OwnerRejectionDecision` and `OwnerReturnReview` inputs themselves must still come from a validated control layer. Lack of JSON parsing is a useful type boundary, but it is not proof of the authenticity of the owner or the quality of the check.

## Why can't we just remove all restrictions?

Let the failure create grant state #2, then another check changes it to #3. Even if the analysis of the first failure confirmed the independence of the result, it **does not own state No. 3**. This restriction is not removed. This also works for the same reason: the newer row already represents a different state.

If the line exactly matches the stored pair and the independence is confirmed by trusted evidence, it is allowed to restore the previous validity, but with a new line number: `1 → 2 → 3`, and not `1 → 2 → 1`. Open snapshots of consumers who read #1 or #2 require a new evaluation. There is no way to quietly fix their dependency number.

An initially unresolved or revoked grant is not included in the denial's own time limits. Even a matching link to the reason does not allow it to be opened. General stop, wait for resource, restore, wait for owner and prohibition of another event are also not removed.

A fourteenth reason has been added to the matrix. All 16,384 combinations of reasons for 11 classes of operations were checked, as well as all combinations of withdrawing only your wait. Separately, 8192 combinations of previous reasons were checked for the initial acceptance of refusal.

## Bounces and repeat messages

The other candidate, area, or final review is rejected. The main work must match the agreed graph view, and the expected line version must match the current one. Number overflow does not create a partial result.

Repeating a message or a new failure when there is already a pending failure does not overwrite the existing entry. While this is a closed failure of a pure function, **not durable replay detection**. The future C-03 must return the recorded receipt upon exact repetition and reject the modified body under the same number.

If the wait record is lost and its cause remains, execution remains prohibited. A new event does not automatically gain the right to remove the previous ban. If there is an entry, but there is no exact reason for it, a general check reveals an inconsistency. Restoring such damage requires verified saved events and has not yet been implemented.

## Tested cases

13 groups have been added to `tests/cases/owner_rejection.rs`: saving one’s own and someone else’s state; prohibition of old acceptance before analysis; lack of the right to bypass an unregistered refusal; repetitions; exact connections and versions; outdated evidence; overflow; combinations of reasons; foreign or newer line; coincident cause without its own pair of states; targeted withdrawal; strict form of reason; new grants in the changed column.

The previous 12 return checks have been updated to the full path "accept failure → parse → return". In the linked example, both consumers first receive a ban of unknown influence, then the proven independent output is restored with a new version. The affected consumer remains blocked. Three new compilation checks prevent JSON from being parsed into a pending record, its state pair, and a ready plan.

General result and fingerprints of specific sources: [STATUS.md](STATUS.md). All data is artificial; There were no tests of actual power loss or operating storage.

## Checking a delegated proposal

Kimi proposed a starter module. The core agent fixed mismatches with valid types, overflow handling, and missing checking of the exact state of work in the graph; connected the module with a direct life cycle, return and independent tests.

DeepSeek offered three observations. They were not confirmed:

1. Removal of the missing own cause is already prohibited by the general `check_root_basis`; added direct negative test for both return and acceptance.
2. A reason left without a record does not intentionally allow automatic acceptance of a new refusal or continuation of execution. This is a damaged state for future restoration, not a reason to assign someone else's prohibition to a current event.
3. Refunds do not go through `AcceptCompletion`: it has a separate linked plan. The existence of a path with a pending ban is verified by tests, including combinations of other reasons. The inspector assumed a missing call.

This does not prove the absence of other errors. External opinion is not applied without verification against code and reproducible verification. OmniRoute settings have not changed.

## What's not ready yet

Real certified owner input, single durable record and restoration; actual semantic verification of reason; branch of the explanation “the condition is fulfilled differently” without taking it for the owner; change of goal; creation of repair units and limited new attempts. These features are not replaced by empty promises in the code, but are clearly left in the development plan.
