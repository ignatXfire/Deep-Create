# Return the total to the owner and influence other work

Date: 2026-09-11. A partial net contract `14.OwnerReturnPlan/v2` has been implemented. Basis: document 07, L-10; document 14, §7.2; document 24, §4.5. The original agreements have not been changed. The second form requires an already registered refusal from [OWNER_REJECTION.md](OWNER_REJECTION.md); a previous direct call without such an entry is now rejected.

## What does the code do now

If the owner's rejection of the exact final candidate has already been registered through `plan_owner_rejection`, and the reason analysis is accepted as requiring correction **within the existing target**, `plan_owner_return` prepares the associated change:

1. The main work is returned from pending acceptance to execution. The line number is increased by one.
2. The old candidate and its final check are removed from the active fields. The complete previous inspection with the candidate remains in the plan as a historical basis, along with the owner's decision and analysis of the reason.
3. For the provided results of this work and the current goal, the impact of failure is tested. Obscurity does not count as lack of influence.
4. Only the targeted expectation of this particular refusal is removed; its complete record is stored in the plan. Other restrictions, rights, obligations, previously accepted packages and expenses are not reset. The return itself does not trigger anything and does not pay for anything.

This is a single recording proposal, not an already recorded event. This path is not yet connected in the application window.

## Precise path and responsibility

Trusted event of the owner → `plan_owner_rejection` and a waiting record → an accepted analysis of the cause and its impact → an agreed upon view of the graph → `plan_owner_return` → a unified plan for the state and admissibility of results → a future general record C-03 → a separate choice and permission for further work.

| Part | What is he responsible for |
|---|---|
| `OwnerRejectionDecision` | Owner, Reason, Candidate, Final Verification Event Numbers and Exact Target Area |
| `OwnerReturnReview` | Linking the analysis to the same failure; expected graph and target versions; existing requirements that need to be corrected; assessing the impact on provision |
| `PublicationImpactAssessment` | The exact provision and version of its admissibility; influence, obscurity or reference to proof of independence |
| `check_root_basis` | Common to the forward and reverse paths, checking the scope, expected string, and consistency of the candidate with the final check |
| `plan_owner_return` | Checks the listed connections and prepares the entire plan without changing the input data |
| `OwnerReturnPlan` | New work state, all read grant states, only required replacements, expected versions and retained history |

Plans and trusted inputs do not have JSON parsed. This excludes such a direct path of turning the text into a finished record, but **does not certify the owner and does not prove the veracity of external verification**. The corresponding boundaries C-05/C-08/C-12 have yet to be implemented.

## Impact on related work

| Initial provisioning state | Analysis | Proposed Change |
|---|---|---|
| Indeed | Impact confirmed | Unresolved state, reason link, next string version |
| Indeed | Impact unknown or review record missing | The same restriction until further clarification |
| Indeed | There is trusted proof of independence | The status and version do not change, but are included in the reading conditions |
| The exact unchanged time constraint created by this particular failure | Trusted Proof of Independence | The previous validity is restored with a new, increased line number |
| Originally unresolved, someone else's changed or revoked | Any of the specified outcomes | The previous reason and version are retained; no automatic recovery |

The check takes place across all submissions of the current work and purpose in the submitted agreed column, and not just those listed by the reviewer. Omitting an influence entry therefore does not leave the grant valid. The internal dialing is organized by numbers; Duplicate assessments, other people's submissions and incorrect versions will be rejected.

For example, Job A provided two results. B uses the first, C uses the second. Until the denial is resolved, the impact is unknown: both grants are temporarily unresolved. The influence on the first and the independence of the second were then confirmed. After a full refund offer, B is left with `PublicationUnresolved`; B may receive a new valid basis after the precise time limit is lifted. Its line number is incremented; An old snapshot that was issued cannot be quietly corrected. The original bytes of both results, their checks and obligations are not overwritten. This related script is executed in a test.

Submissions of another purpose or job are not automatically revoked. If the cause really affects them, the control layer is obliged to conduct a separate analysis of the expanded area of ​​influence; this module does not prove their independence. The completeness of the transferred graph remains the responsibility of the future storage.

## Why return is not the same as resuming execution

The semantic stage and prohibitions are different parts of the state. You can get an accepted decision to return in the event of a stop, recovery, or an unknown external outcome; It is still not possible to perform prohibited actions after this. All 16,384 combinations of the existing 14 reasons have been checked: only the own targeted failure expectation is removed, the remaining reasons are preserved and do not resolve the effect.

The owner's general expectation is not automatically deleted: this general attribute does not store an exact association with the event. The new `OwnerRejectionPending` stores such a connection and is removed only by a matching pending entry. This does not allow clearing other reasons or someone else's expectation.

There is no requirement that a rejected candidate remain fresh: otherwise changing the proof might make the old result both unacceptable and unreturnable. At the same time, **the parsing of the reason itself** is associated with the current versions of the graph, the goal, and the explicitly verified grant states. Someone else's target, unknown requirement or outdated estimate will not be accepted.

## Future shared record obligations

Writer C-03 must in one operation:

1. Re-check the scope and expected main work line, graph version and target line.
2. Check **all** rows from `publication_reads`, including states that are not modified by this plan. You cannot check just a list of records.
3. Consistently replace the work state and the specified grant states, save the returned candidate, final check, refusal and reason analysis.
4. Preserve the connection with a single owner event, eliminate command repetition and reuse of global numbers.
5. Confirm the result only after the required control point.

If two operations collide in version, one must be re-evaluated. It is impossible to partially apply only the return of work, leaving the dependent consumer with the old admissibility. A clean test collects the entire proposed graph in memory; it is not yet testing a transaction, a crash, or a power outage.

## Checks

`crates/dc-domain/tests/cases/owner_return.rs` added 12 groups of checks connected to existing life cycle checks:

- successful return, absence of current old candidate and preservation of accurate historical basis;
- confirmed, unknown, missed and proven independent influence;
- preservation of initially unresolved, foreign and revoked states; targeted removal of only your own unchanged time limit;
- all combinations of locks;
- substitution of event, cause, purpose, candidate or final test;
- empty, unknown, alien, repetitive and excessive lists of requirements;
- old versions of the root, graph, goal and provision, including with declared independence;
- unknown, repetitive and excessive impact estimates;
- version overflow without partial input change;
- return of an outdated candidate and prohibition of accepting him with the previous decision immediately after the return;
- invalid stages and inconsistent fields;
- related example A → B/C: after parsing, the affected consumer remains blocked, the proven independent one gets the opportunity to collect a new valid basis.

Additionally, three compilation checks prohibit parsing JSON directly into a solution, accepted parsing, and ready plan. The numbers and exact results of the last general pass are in [STATUS.md](STATUS.md).

## What is intentionally not declared ready

The original rule of rejecting **before** accepted parsing is now implemented as a pure rule in `owner_rejection.rs` and linked to this module. A true authenticated owner channel and long-lasting recording are not yet available. The entire rejection order is therefore still not declared as a finished application.

Separately remain: the actual test of cause and independence; the response “the condition is already met otherwise” without automatically being accepted as the owner; change of purpose; targeted removal of expectations; limited new attempts; creation of repair units; single long-lasting recording and replay protection. Once returned, the old candidate number cannot be reused; The global registry has not yet been implemented, so there is no complete protection across the entire application.

## Delegation and proposal review

Kimi has prepared a narrow module proposal. The integration fixed a missing check for ownership of repair requirements and an incomplete state replacement that would have left the same candidate fields. Additionally, parsing versions are linked to the graph and target, the target string is saved as a condition for a future record, and the existing verified version arithmetic is reused. Tests are written and executed by the main agent.

DeepSeek's limited review reported no additional defects. This is the opinion of one limited entry reviewer and is not proof of the absence of errors. After it, end-to-end verification of two consumers was added. OmniRoute settings and Hermes product code have not been changed.
