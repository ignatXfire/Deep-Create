# Lessons from a busy Deep Create session

The session is useful not because it “has already proven readiness,” but because it showed real mechanisms and real failure modes.

## What worked

### Orthogonal proof lanes
In one wave, four subagents received different tasks: live E2E, crash recovery, installer validation and Stage 3 persistence/schema. This decomposition is good: independent resources, different proofs, low probability of writing the same thing.

### Parallel DAG proof
Later, a diamond DAG was fixed, where independent branches started simultaneously. This is exactly the type of parallelism that needs to be scaled: dependency-aware, not just “run 100 coders.”

### Real runtime gates
The session contained tests of live dispatch, OS kill/restart, installer and workspace integration. This is much stronger than a text self-report agent.

## What turned out to be dangerous

### Historical green ≠ current acceptance
Even after a large green suite, subsequent work directly stated that the old readiness percentages are not current acceptance evidence. This means that the dashboard must version-bind each metric to candidate SHA and gate version.

### Independent reviewer can also hallucinate
A separate review artifact was rejected after readback: the claims contained non-existent files/symbols and an incorrect git delta. Conclusion: model diversity reduces correlated error, but reviewer receipts should also be checked by machine.

### One writer per WIP
In the session, the duplicate support card was gated as long as there was one continuation writer on the corresponding WIP. This is the correct control against double mutation.

### Worker labels are not proof
Counts/hashes were obtained from gate receipts, and not from “DONE” tags. This should be made a universal rule in the new factory.

## How to transfer to 700k factory

1. Save dynamic workgraph and blocker routing.
2. Increase the number of **independent lanes**, rather than writers, per task.
3. All reviews binding to exact SHA/diff/path existence.
4. Old reports are historical leads, not runtime truth.
5. Product-ready status is calculated from the live gate matrix, and is not specified in text.
6. For each WIP semantic authority there is a maximum of one active mutator.
7. More attention to build/test/evidence capacity than to the number of model responses.
