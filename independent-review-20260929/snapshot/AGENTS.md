# Deep Create - implementation rules

This is a separate personal application, not a Hermes modification. Only the designated candidate/worktree within `D:/Projects/DeepCreate\.local\hermes-organization-20260916` is allowed for this copy. The original project outside this organization is read-only. Do not modify the adjacent `Draft`, Hermes profile, vendor keys, or user archives. Organizational agreement: `../ORGANIZATION.md` regarding integration candidate; worktree members receive its absolute path in Kanban. Do not run Codex CLI or delegate_task; work is organized through Hermes Kanban.

## Contract source

- `docs/technical/00_BRANCH_NAVIGATOR.md` - enter saved contracts.
- 07 - decisions made by the owner; 13 - hierarchy and boundaries; 19 — C-01—C-16 and dependencies; 20 - sequence and evidence; 21 - traceability; 23–26 — latest clarifications.
- Read the entire required contract and only the necessary related sections, not the entire research corpus. Do not provide a description of the test for its execution.
- Updates after external verification: `docs/implementation/REVIEW_20260910.md`, `TOPOLOGY_REVISION.md`, `LIFECYCLE_GATES.md` and `contracts/contract-map.json`. The 21 original documents saved were not rewritten retroactively; partial type is not equal to full canonical notation.
- Maintain branching, several long works, complex connections and self-development. Do not replace them with a simple chat for the sake of an easy result.

## Code and boundaries

- Rust - load-bearing layer; Tauri - thin shell; TypeScript/React - representation. Permissions are not granted by model text or JavaScript.
- C-01 clean: no network, files, clock, environment, Windows, base. No `unsafe` outside `dc-windows`.
- The main graph and expense have one writer. Queue acceptance does not equal confirmed commit. The unknown outcome is not repeated blindly.
- Do not create empty packages/functions of future stages. The significant function has a C-contract, calling, closed failures and verification.
- Work only in designated files; save other people's changes. General types should be agreed with the main agent.
- All tests and experimental data are artificial. Real connections/data are only allowed after the appropriate steps.
- Place assembly products, temporary files and local test data on D. Do not change the global environment or user settings for the sake of testing.

## Submitting changes

For each completed slice, indicate: what works, how to reproduce it, what negative cases have been tested, what has not yet been implemented. The process test should not be declared to be a real power outage test. The actual promotion register is `docs/implementation/STATUS.md`.
