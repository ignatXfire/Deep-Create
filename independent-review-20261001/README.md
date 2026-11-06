# Deep Create - fresh independent audit 10/01/2026

**Verdict:** The project has changed significantly, several previous defects have been corrected. Full operational readiness remains unconfirmed; found current P1s in identities, persistence, content acceptance, dependencies, unknown and OS boundary.

Snapshot: `f7444d158fb9900217e8a62d0d0c53cbff2f5dd3` plus two original dirty edits. 290 selected source/contract/context files have been saved. 60,452 files outside 375 excluded subtrees were listed by metadata; 0 enumeration errors. This is not a line-by-line check of each file.

Latest results:

- TypeScript: exit 0.
- UI: 50 passed, 3 failed, exit 1; reasons and limitations are described.
- Rust domain/protocol/store/watchdog: 376 passed, 0 failed, exit 0.
- Independent probes: 6 passed, exit 0 - five violations were reproduced, one correction was confirmed.
- Full workspace/native release/clean VM/30-min soak were not performed in this pass.

Major new proven defect: machine verifier returns Passed for the result of **another unit**, with an arbitrary non-zero fingerprint and no evidence checking criteria. The erroneous schema 10→3, loss of the second entity connection evidence, signed journal ordering, and the ability to move the impersonation guard between threads were also confirmed. These results are not reports of actual exploitation/loss of user data.

## Read in order

1. [01_CHANGE_REVIEW.md](01_CHANGE_REVIEW.md) - re-checking all 12 previous questions.
2. [02_FINDINGS.md](02_FINDINGS.md) - current violations, places in the code, solutions and negative checks.
3. [03_EVIDENCE_AND_SESSION.md](03_EVIDENCE_AND_SESSION.md) - real results and analysis of session-20260930.
4. [04_ARCHITECTURE_READINESS_PLAN.md](04_ARCHITECTURE_READINESS_PLAN.md) - mechanisms, readiness and further order.
5. [05_DEPENDENCIES_AND_EXTERNAL_SOURCES.md](05_DEPENDENCIES_AND_EXTERNAL_SOURCES.md) - reasons for choosing dependencies and primary sources.
6. [06_HANDOFF_AND_SCOPE.md](06_HANDOFF_AND_SCOPE.md) - reproduction and unverified volume.
7. [07_ORGANIZATION_AND_PROVENANCE.md](07_ORGANIZATION_AND_PROVENANCE.md) - copies and status of evidence of the organization.
8. [evidence/results.json](evidence/results.json) - hashes, exit codes and the actual immutability of the snapshot.

**My opinion on the readiness of the full plan: approximately 25-40%, low confidence.** There is no measured percentage and no reliable date yet. A map of accepted requirements and measured integration speed are needed. The result of this audit does not allow access to valuable accounts: the necessary chain of custody has not been proven.

The ZIP contains this package being tested and the selected source snapshot; it does not replace the full history of all .local candidates. Old reports in snapshot are statements/context, not automatically confirmed facts. User messages and session statements are stored with indexes; export automatic messages with role=user are separated heuristically.
