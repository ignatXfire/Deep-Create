# New branch navigator Deep Create

**Updated:** 2026-09-09  
**Borderline:** stand-alone Deep Create application, not a remake of the current Hermes.

## Where to start now

1. [Analysis of external criticism and clarification](./23_EXTERNAL_CRITIQUE_AND_REFINEMENTS_20260908.md) - a solution to each of the twelve comments, changes, re-checking and the boundaries of what has been proven. [Previous total](./22_DESIGN_REVIEW_20260907.md) saves the history of the stage 2026-09-07.
2. [Map of code, functions and dependencies ](./19_CODE_FUNCTION_AND_DEPENDENCY_MAP.md) - who calls each significant function, what it receives, changes and returns; how a change in one part affects another.
3. [Matrix of solutions, code, checks and beacons](./21_DECISION_CODE_VERIFICATION_MATRIX.md) - 51 numbered solutions and 10 additional groups of requirements that cannot be lost during implementation.
4. [Procedure for creation and proof of readiness](./20_IMPLEMENTATION_ORDER_AND_READINESS_EVIDENCE.md) - necessary steps, checkpoints and prohibitions on premature transition.

## Technical agreements

| Document | The main question |
|---|---|
| [13. Technical Constitution](./13_TECHNICAL_CONSTITUTION.md) | Where is the boundary of application and who owns each kind of truth? |
| [14. Data and recovery](./14_DATA_STATE_AND_RECOVERY.md) | What do we write down and how not to lose the meaning, rights and expenses after a failure? |
| [15. Context, memory and instructions](./15_CONTEXT_MEMORY_AND_INSTRUCTIONS.md) | What exact data does the model leave and how is its response separated from local evidence? |
| [16. Execution and verification](./16_EXECUTION_MODELS_AND_VERIFICATION.md) | How are connections selected, branches created, and results verified? |
| [17. Rights and Actions](./17_PERMISSIONS_SECURITY_AND_ACTIONS.md) | Why can't any mode or skill bypass the consequence boundary? |
| [18. Beacons and stop](./18_BEACONS_JOURNAL_STOP_AND_RECOVERY.md) | How to see an important fact, stop work and not receive the same notification all the time? |
| [24. Work links and common work areas](./24_WORK_DEPENDENCIES_AND_SHARED_WORKSPACES.md) | what edition of someone else's result is used and how to coordinate changes to real shared objects? |
| [25. Resources and Commit Queue](./25_RESOURCE_ALLOCATION_AND_COMMIT_QUEUE.md) | How do works get progress without being crowded out in a circle and how does waiting differ from acceptance? |
| [26. Combinations of states and development](./26_COMPOSITE_STATES_AND_EVOLUTION_TESTING.md) | How are compound failures, test reliability, and the actual usefulness of improvements tested? |

## Basis of decisions - saved documents

- [07. User's answer card](./07_OWNER_DECISIONS.md) - accepted answers and detailed additional requirements.
- [08. The architecture of a separate Deep Create](./08_STANDALONE_ARCHITECTURE.md) - independence, boundaries and a reading bridge to Hermes.
- [09. Interface Study](./09_INTERFACE_RESEARCH.md) - Map, List, Rights, Stopping, Human Errors and Accessibility.
- [10. Risks and hidden places](./10_RISKS_AND_HIDDEN_BOUNDARIES.md) - initial risks and limitations of independent change.
- [11. New direction and plan](./11_DESIGN_DIRECTION_AND_PLAN.md) - a coherent explanation of the entire system at the level of the adopted architecture.
- [12. Previous completeness check](./12_COMPLETENESS_REVIEW_20260904.md) - historical report of its stage; not proof of the readiness of the current implementation.

Old studies are preserved as the origin of ideas. The description of Deep Create inside Hermes is inferior to the accepted standalone application. Documents 13–21 and 23–26 clarify the technical mechanism, but do not allow silent changes to the owner's decisions from 07 and the current boundaries of 10/11. The rule for resolving contradictions is specified in document 13; 22 - historical report.

## Current status

- Architectural direction and previously accepted custom boundaries: fixed.
- Technical agreements, code map and creation sequence: prepared.
- Structural connections: checked by [local tool](./checks/verify_design.ps1); [saved result 2026-09-08](./checks/design_check_20260908.json) refers to specific document fingerprints. This is not proof of the semantic completeness or readiness of the code.
- Unverified engineering properties: initial list in 22, additional scenarios and tolerance limits in 23–26 and 20 §19.
- Re-checking clarifications: six detected inconsistencies have been corrected; information about the verification and its limits - 23 §9.
- Individual application product code and end-to-end testing: not yet created or completed.

The next step is minimal self-assembly and a verifiable implementation according to document 20. The presence of documentation does not give the right to declare the application working.
