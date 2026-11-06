# Documentation map

The documents preserve the research history. Earlier status statements describe their own date and do not replace `../RESEARCH_STATUS.md`.

For the main research thesis, start with [the evidence-carrying feedback loop](../publication/CYCLIC_WORK.md), then inspect [the claim-to-code evidence map](../publication/EVIDENCE_MAP.md) and [the proposed efficiency evaluation](../publication/EVALUATION_PROTOCOL.md). These distinguish concrete component mechanisms from unmeasured end-to-end benefits.

## Architectural foundation

- [Navigator](technical/00_BRANCH_NAVIGATOR.md).
- [Owner decisions](technical/07_OWNER_DECISIONS.md).
- [Technical constitution](technical/13_TECHNICAL_CONSTITUTION.md).
- [State and recovery](technical/14_DATA_STATE_AND_RECOVERY.md).
- [Context, memory, and instructions](technical/15_CONTEXT_MEMORY_AND_INSTRUCTIONS.md).
- [Execution, models, and verification](technical/16_EXECUTION_MODELS_AND_VERIFICATION.md).
- [Permissions and actions](technical/17_PERMISSIONS_SECURITY_AND_ACTIONS.md).
- [Beacons, stop, and recovery](technical/18_BEACONS_JOURNAL_STOP_AND_RECOVERY.md).

## Requirements and implementation

- [Code and dependency map](technical/19_CODE_FUNCTION_AND_DEPENDENCY_MAP.md).
- [Implementation order and readiness evidence](technical/20_IMPLEMENTATION_ORDER_AND_READINESS_EVIDENCE.md).
- [Decision/code/check traceability](technical/21_DECISION_CODE_VERIFICATION_MATRIX.md).
- [Work dependencies and shared workspaces](technical/24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md).
- [Resources and commit queue](technical/25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md).
- [State combinations and evolution](technical/26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md).

`implementation/` contains narrower notes on lifecycle, protocol codec, scheduling, authority/resources, owner return/rejection, and the contract map. Compare them with actual source callers and the independent audit.

## Evidence and history

The latest included independent review is [October 1](../INDEPENDENT_REVIEW_2026-10-01.md), with [detailed evidence](../independent-review-20261001/README.md). The [September 29 review](../INDEPENDENT_REVIEW_2026-09-29.md) provides earlier context.

Root evidence/status and wave reports are preserved, including conflicting author claims. An author's completion message is not independent acceptance. Private data has been redacted, and historical hashes do not authenticate transformed public files.
