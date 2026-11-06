# Transfer to next independent reviewer

## Read sparingly

1. README.md and 01_FINDINGS.md of this package.
2. evidence/results.json, evidence/baseline.txt, snapshot-manifest.json.
3. Only the code of a specific card and related sections of contracts in snapshot/docs/technical.
4. 02_ARCHITECTURE_AND_PLAN.md - work order and boundaries of personal opinion.

The snapshot contains the main code and selected contracts, not all of the organization's historical working copies. inventory.json stores the map of the remaining paths for address reading in the source directory. Do not declare a historical document missing from the ZIP as non-existent in the original project.

## How to check comments

First, check the current SHA and hash of the desired file with the snapshot. Check each comment separately: reproduced / corrected with new code / contract changed / insufficient evidence. Do not summarize identical functions of different copies. Do not transfer data from the old report to the verified column.

`probes` is a separate Cargo package that uses the real dc-domain/dc-store from the snapshot. PASS here means **defect reproduced**, not “system OK”. After the repair, you should invert the expected result of the required regression check and link it to the new version of the product. Do not retroactively change the original image for the sake of a green run.

Run with the MSVC process environment from tools/Enter-BuildEnvironment.ps1 of the original project, CARGO_TARGET_DIR/TEMP/TMP/AUDIT_TMP on D. Commands for this audit:

```powershell
cargo test --manifest-path <audit>/snapshot/Cargo.toml --locked --offline -p dc-domain -p dc-protocol -p dc-store -p dc-watchdog --features dc-protocol/test-peer-constructor,dc-store/sqlite
cargo test --manifest-path <audit>/probes/Cargo.toml --offline
node node_modules/typescript/bin/tsc --noEmit -p tsconfig.json
node node_modules/vitest/vitest.mjs run
```

For the last couple, cwd was the main project. For Rust, the saved snapshot was checked. Don't literally transfer these commands to someone else's machine without setting up local paths. Do not run the entire workspace test blindly: F12.

## What remains untested

- Each of tens of thousands of .local files and all historical chats/JSONL; Contents of all reported fixes by different teams.
- Current Hermes Kanban and the complete state of the organization: processes and their age do not replace board/evidence; the audit did not interfere with its management.
- Real Tauri window, installation in a clean VM, full UI→backend round-trip.
- Full default-feature workspace build, all integration tests, real models/accounts.
- Kill/restart in all failure windows, power failure, long-term load, memory, performance and economy.
- Real OS security sandbox, migrations/updates, full dependency vulnerability/license audit.
- Complete tracing of all decisions 07 and all points C-01-C-16.
- Full story of ChatGPT “AI Father”: an inaccessible story cannot be considered read; existing local agreements are an accessible source of design.

Further small portions: (A) desktop identity/IPC and real artificial script; (B) durable command boundary/unknown/budget; (C) OS isolation; (D) context projections and several long works; (E) self-development and evidence of accepted integration. Each package contains 1-3 invariants and negative checks, rather than a massive rewrite of the architecture.

This independent review must not be turned into permission to edit an adjacent Draft, Hermes profile, keys or archives. Corrections of the organization should be carried out in the designated candidate/worktree, taking into account the current AGENTS.md and ORGANIZATION.md. In this audit, only new report/synthetic check files are changed; the operating organization was not reconfigured.
