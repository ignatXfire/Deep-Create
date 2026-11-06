# Maintenance of various jobs: implemented arithmetic part

Basis: Document 19 C-01 and Document 25, Sections 4.1, 4.3 and 9.1. Code: `crates/dc-domain/src/scheduling.rs`. Checks: `crates/dc-domain/tests/scheduling_contracts.rs`.

## Responsibility of each function

| Function | Input and result | Why is it needed and who uses it |
|---|---|---|
| `validate_scheduling_policy` | Proposed rule version and trusted setting boundaries → checked immutable value or closed error | The future C-05 accepts the rule change only after this review. Now called for testing. Does not validate the owner event and does not save the rule. |
| `compare_root_service` | Two valid candidates and a proven rule → exact order, reason and numerical bases | Future C-06 compares normal moves after strict quality/rights selection and after protected wait processing. Now called for testing. Doesn't launch an attempt. |

The remaining functions inside the file only select the weight of a closed class or add three values. They do not have a separate record, event or process.

## What is being compared

For each candidate, three values are added in one dimension: confirmed servicing of completed attempts in the current window, conservative servicing of **all** incomplete attempts, and the upper cost of a new move. The latter is equal to the number of required positions of performers multiplied by the forcibly limited duration of the attempt.

After this, the fractions `обслуживание / вес владельца` are compared. Numbers are not converted to floats: they use checked integer sums and cross-multiply into the broader integer type. If there is a tie, the older waiting number is selected, then the stable job number. Zero, unknown, or infinite duration are not represented by valid input. An overflow of the amount or an unprovable ceiling gives a refusal, not a zero value.

Monetary price, response quality, text urgency, and perceived usefulness are not mixed into this metric. Research remains a class in its own right, and the ultimate benefit of current work does not give it absolute priority.

## What is being checked

Five tests check an example from a contract, the finiteness of the advantage of the current work, an exact comparison beyond the JavaScript precision bound, order under equality, an unknown version of the rule, exceeding weights and try bounds, and overflow. Additionally, 18 artificial combinations are sorted out in pairwise and triple comparisons: the order must be opposite when rearranging a pair and consistent for a chain of three elements.

The module underwent a separate independent agent analysis; No specific defects were found in the declared arithmetic domain. Executable results relate to a specific edition and are recorded by the general `Verify.ps1`.

## What is intentionally not here

This is **not the entire planner**. Checking real prices, quality of models, origin of units, absence of double counting, observation time, aging, protected next move, recording queue and moving the waiting number after a certified launch must still be implemented by the control layer. It is impossible to sort all works with this function and call it ready-made justice. Correct arithmetic without these mechanisms does not guarantee the progress of large, long-term work.
