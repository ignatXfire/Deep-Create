# One version of graph relationships: owner, write and check

Addition dated 2026-09-10 to documents 14 §4.9 and 24 §§3–5. This is a refinement of the first version, not a rejection of multiple works, complex dependencies, or independent research.

## Which space is confirmed

Previously, it was required to compare the graph version, but the location of its storage was not clearly named. A single writer by itself does not save: two pre-checks can read one old state and separately approve opposite arrows. The second entry should detect a change in the base and not just wait in line.

## Solution for the first version

In the main database of each installation there is exactly one row `graph_topology_head`, key `singleton=1`. It is owned by C-03 within the control process; the window, the model, the performer and the intermediary do not change it. The line contains the identity of the installation and the version of all accepted structural changes to the graph. It is not an event number, a version of an individual job, a generation of a process, or a permission to take an action.

The initial version is 1. Each accepted change to the structure increases it by exactly one. Overflow prohibits modification and requires parsing; zero and zeroing are not allowed. The full 64-bit unsigned version is written in eight bytes, most significant first: a regular SQLite integer does not cover this entire range. The SQL fragment is located in `contracts/topology-head.sql`; the conversion in application storage should preserve the exact bytes and not go through a JavaScript number.

The version is incremented when units/revisions are created, replaced, or canceled, parent dependencies, required work relationships, conditions, and goals/commitments that change the composition of potential expectations. Changes in future bindings that could change the manufacturer's choice also pass this boundary. A simple new consumption observation, revoking a right, writing a text, or changing the readiness state of a result does not automatically increase it: they have their own dependency snapshot lines and guardrails. If a new command can change the topology but does not have a rule registered, it is not allowed to be written.

The check should see the full potential wait area, including the conditional edges and the wait for the owner's decision. One global version does not make an incomplete graph complete. C-03/C-05 are required to collect a consistent view and not accept the model's chosen subset as the canonical region. Until this reader is implemented, neither Rust's public structure nor a successful pure function test counts as proof of read completeness.

## Exact order

1. The trusted reader receives the consistent state, installation identity, and version 7.
2. C-01 checks the proposed graph. `prepare_topology_change` binds the tested immutable candidate to the expected line and the next number 8. The plan has no parsing from JSON and no commit command.
3. A single writer starts the transaction. Re-verifies all significant grounds, existing rights/protections and restrictions in accordance with the main agreement.
4. In the same transaction, a conditional head replacement is performed: `WHERE singleton=1 AND installation_id=:installation AND revision=:expected`. Exactly one line must be changed. Subject changes and events are recorded along with it. An error in any step cancels the entire transaction.
5. If the head is already 8, the candidate based on 7 receives `GraphVersionConflict`. You can't just substitute number 8 into the old plan: you need to read the area again and check the merged graph. The opposite arrow would then give `Cycle`.
6. The success of the main transaction does not yet equal the final confirmation: the previous checkpoint order at the guard is maintained, after which confirmation, dependent action and accepted result are allowed in the window.

The global head deliberately creates additional conflicts between unrelated structural changes. It does not stop parallel computations and does not make any observation a global entry in the graph. Subdividing the head into independent regions is permitted later only after separate proof of the completeness of the region and the absence of unnoticed new edges.

## What has already been accomplished and what it does not prove

- Rust implements `TopologyHead`, opaque `PreparedTopologyChange`, preparing and checking exact base with failure on overflow. The candidate is held by an immutable reference.
- Five executable real SQLite checks check for old number, foreign installation, invalid string form, atomic rollback and exact numbers above JavaScript/signed SQLite range. Two connections read the synthetic database file; the order of the contending sentences is specified explicitly.
- Edge/Event Challenge Tables - Minimum Experience Tables only. This is **not an implementation of C-03**, nor is it a test of full simultaneous processes, nor is it a test of power failure.
- Checks use the built-in experimental `node:sqlite` in development tools only. It is not added to the application and is not selected instead of the Rust store. In its absence, the general test pass stops rather than silently passing the experience.

The following mandatory check when implementing C-03: same script via real `commit_transition`, consistent reader, full row set and event; then failures before/after the master record and before/after the guard checkpoint. Existing experimental results do not replace these checks.
