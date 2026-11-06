# Pure matrix gate × operation

Section C-01 dated 2026-09-10. Implementation: `crates/dc-domain/src/work/lifecycle.rs`; tests: `crates/dc-domain/tests/lifecycle_contracts.rs`.

Reasons: 14 §4.7, §§7–9; 18 §§7–9; 24 §2.1–2.2, §8–10; 26 §2–3 and §6.1. Section 8 of external review is accepted as a closed machine table requirement. His example of permitted reading during recovery is not carried over literally: Treaty 14/18 allows, without continuation, only pre-registered limited local diagnostics, not previous units of reading and reasoning.

## What is implemented

`GateOperation` - closed classes of operations; `GatePolicy` exhaustively matches each existing `BlockingReason` table row. Adding a variant of any enumeration requires processing in `match`. External text is not deserialized into the new admission policy.

`evaluate_gates` checks the conjunction of all passed causes and all classes of the compound operation. Result `GateAssessment` contains only blocking reasons with original typed details. An empty class list is rejected. `is_clear()` only means the absence of a prohibition on the part of these gates, and not an issued right.

You need a complete consistent installation → root → unit view, valid permissions for all areas, term, fencing, resources, source validity, and a valid entry. None of these responsibilities are replaced by the matrix. The check does not verify the completeness of the transmitted view. The same `BlockingReason` in several areas can collapse into one diagnostic cause; this is not a canonical registry of individual gates and not a command to remove them.

## Table

`1` - only this gate does not prohibit this class; `0` - prohibits. Designations: R - normal reading; T - reasoning; P - planning; E - consequence; C—cancel; D—trusted local diagnostic reading; Rc—service reconciliation; F - preservation of the observed fact; O — acceptance of the owner’s decision; S - semantic promotion of the result; A—Exact final acceptance by owner.

| BlockingReason | R | T | P | E | C | D | Rc | F | O | S | A |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Recovery | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| IntegrityFailure | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| ResourceUnavailable | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| StopRequested | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| UnknownOutcome | 1 | 1 | 1 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| OwnerRejectionPending | 1 | 1 | 1 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| WaitingOwner | 1 | 1 | 1 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 1 |
| AwaitingPublication | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| PublicationUnresolved | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| PublicationWithdrawn | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| OwnerAcceptancePending | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| ConditionUnknown | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| UpstreamCancelled | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |
| DependencyPending | 0 | 0 | 0 | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 0 |

The causes of addiction relate to the exact dependent unit, not to the extraneous work. An unknown or revoked mandatory source does not become a valid basis for its normal operation.

D, Rc, F and O are closed service path classes, not “access to all”. D does not denote reading of user material by a veteran artist. Rc does not allow arbitrary network: accurate secure network reconciliation requires its own valid law, disclosure and expense control. The presence of a gate should not block the very possibility of reliably accepting the fact of an account, outcome, loss of a process or a decision to stop; the actual recording may still not take place. The matrix does not issue a false ACK when the disk is faulty.

The composite action checks all applicable classes. For example, `Plan + Effect` is prohibited under AwaitingOwner, although scheduling alone is not prohibited. `ReceiveOwnerDecision + AcceptCompletion` is prohibited during Recovery, although obtaining the owner's decision is not prohibited. The required normal reading or effect cannot be hidden under the name Rc/D.

## Lifecycle integration

- `Prepare` and `Start` check R + T + P + E. The existing `WorkKind` describes the expected result, but does not certify that there is no network call, disclosure, or other consequence. Until the exact classification of actions in C-05, the general launch remains closed at any current gate, including `WorkKind::Planning`. The matrix already distinguishes between pure classes, but this slice does not create an execution route for them. The limited `plan_continuation_new_attempt` uses the same set to pass continuation: it is preparing a new line, not starting, and `UnknownOutcome` also blocks it.
- `ObserveSubmission` is first tested as F. On a current attempt at `Running`, promotion to `ResultSubmitted` additionally requires S. If S is denied, only `LateObservation` is returned: no replacement, attempt line advancement, accepted result, or commitment closure. Packet byte retention, authentication, and durable event recording remain the responsibilities of the future C-03/C-05 service path. `ResultSubmitted` itself does not yet constitute acceptance of the result, but it opens a semantic path for verification and therefore is not applied after the ban.
- Cancellation request/confirmation use C; Loss Observation - F. Preserves previous checks for unit/attempt pair form, exact lines, area, edition, and generation. The future generation is rejected; the old generation allows only the fact of observation and does not come to life after the reasons are removed.
- `plan_root_transition` necessarily receives `installation_gates` and combines them with its own gate of the main work. Candidate generation and final validation use S. Final acceptance uses O+A and still checks for consistency of the current candidate, accurate final validation, freshness of dependencies, and verified owner solution.
- The only gate that does not prohibit A by itself is `AwaitingOwner`. This is not a universal owner-command bypass: `OwnerAcceptancePending` someone else's publication, stop, recovery, unknown outcome and other reasons prohibit acceptance. The plan contains `OwnerCompletionAccepted` with the exact owner event, candidate and final verification ID. The scheduler does not remove any causes; clearing a separate wait requires a separate precise connection with the confirmed event, which does not yet exist in the current `BlockingReason`.

Plan - proposed rows and event, not COMMIT. The C-03 is required to repeat the checks at the agreed acceptance point. The plan does not stop the clock, does not serialize the revocation with D/C, and does not prove Windows isolation.

## Factual check

Executed locally on artificial UUIDv7 fixtures:

```powershell
. .\tools\Enter-BuildEnvironment.ps1
rustfmt --edition 2024 crates/dc-domain/src/work/lifecycle.rs crates/dc-domain/tests/lifecycle_contracts.rs
cargo test -p dc-domain --test lifecycle_contracts --offline --locked
```

The initial result of this slice was **15 passed, 0 failed**. After adding the failover and return paths from `OWNER_REJECTION.md` and `OWNER_RETURN.md`, the target set contains 40 tests; the current overall result is in `STATUS.md`. Matrix checks expanded:

- 154 named cells: 14 existing reasons × 11 classes, independent literal table of expectations;
- all 16,384 subsets of 14 causes × 11 classes, that is, 180,224 combinations; exact set of blocking reasons and independence from the order of merging areas;
- all pairs of classes for each reason, an empty list of classes, prohibition of hidden consequences in planning and prohibition of substituting acceptance with the owner input technique;
- all 16,384 combinations of reasons for the main operation and installation in the present function of final acceptance by the owner, with the exact event and storage of the reasons;
- each gate in Prepare/Start/ObserveSubmission/cancel/lost; lack of meaningful promotion of the late package; removing one cause does not remove others and does not restore the previous generation;
- legacy regressions of exact strings, generation, unit/attempt form, A/B candidate, final check, owner event, deprecated dependencies and private command parsing.

Completeness limit: subsets of one synthetic representative of each existing `BlockingReason` variant have been enumerated, not all IDs, version values, real storage states or inter-process schedules. These are pure tests of the table and the current lifecycle, **not** a model of the entire protocol 26, not a C/D/F check, not running a scheduler, not experiencing a process failure or power outage. Acceptance of rejection and return are implemented as pure plans: your address ban is lifted only after an exact entry, the rest are saved. Only a narrow plan for a new attempt after `Blocked + Failed` has been implemented; it is not a resume and does not cover `Lost`/`Cancelled`. True authenticated input, target change, universal canonical cause records, store and executor have not yet been implemented.
