# Continue through a new attempt

Date: 2026-09-12. Implemented section C-01: `crates/dc-domain/src/work/continuation.rs`, public function `plan_continuation_new_attempt`. Tests - `crates/dc-domain/tests/cases/continuation.rs` via `lifecycle_contracts.rs`.

This is not general `resume`, not dispatching, and not writing to the database. The function builds a replacement `WorkUnitSpec`, a new `AttemptRecord`, and the exact reading conditions for the future single writer. It does not read the clock, network, files, environment or storage and has no right to clear a gate, repeat an external call, transfer a reservation, or revive an old attempt.

## Allowed entry

Only one consistent case is supported: the current unit is `Blocked`, its **last** attempt passed by the caller is `Failed`, and the unit has no accepted result. `ContinuationContext` must contain a complete set of grounds: the current `run_generation`, the gate of the installation, the main work and the unit, the expected row versions and a reliable counter of already recorded attempts for this unit. An empty or zero counter is not interpreted as a safe new start.

Before creating a plan, the following are checked:

- coincidence of scope, unit and generation of the previous attempt with unit and the current launch;
- exact versions of the unit and the old attempt;
- excellent identity of the new attempt;
- lack of accepted result;
- `attempts_used + 1` without overflow and not higher than `max_attempts`;
- a complete combined set of gates in the same conservative matrix R+T+P+E that conventional preparation uses. Therefore, `UnknownOutcome`, stop, recovery, resource, dependency and any other protective reason cannot be bypassed by repetition.

## The result and reason for this form

If successful, the old attempt remains unchanged. A new `attempt_id` is created with stage `Prepared`, and the unit receives stage `Leased` and a version one higher. This is intentionally the pair `Leased + Prepared`: in the existing lifecycle `Ready` means no current attempt. The `Ready + Prepared` pair would contradict the already accepted state model and would allow the error to be hidden until the next transition.

The plan also returns the previous row base, the expected saved counter, and the new attempt number. C-03 must, in one transaction, compare all this again with `unit_attempts`, make sure that the old entry is indeed the last one, insert a unique new entry and apply the unit replacement. The success of a pure function does not prove the uniqueness of the ID, the completeness of history, or atomicity.

## What is deliberately not included

- The C-13 still must submit and authorize the resume by the owner; this plan does not create such a right.
- C-05/C-09 must still check valid credentials, freshness, limits, admission and external call immediately before it.
- A lost, canceled or unknown external attempt is not continued in this way: a separate contract for reconciliation and restoration of the fact is first needed.
- The history of the cause is not reduced, the result is not transferred, the expense is not released or written off again.

## Test scripts

The tests guard the exact bases, generation and region; limit and overflow; each incompatible unit/attempt stage pair; someone else's result; gate of all three levels, including `UnknownOutcome`; the immutability of the old attempt; inability to use the old basis to start a new attempt; and repeated gate check directly at `Start`.
