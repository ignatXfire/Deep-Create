# Instructions to the next AI and packet boundaries

Read sparingly: README → 01_CHANGE_REVIEW → specific N-note in 02_FINDINGS → its source files and related contract → evidence/results.json → 04_ARCHITECTURE_READINESS_PLAN. Don't re-read all the old corpuses before every small task.

Package - a verifiable snapshot of the main implementation, documents and history of selected statements. It does not replace a full download of tens of thousands of historical .local files. inventory.json gives their address card with metadata. A candidate missing from the ZIP cannot be considered non-existent in the source directory.

The working snapshot includes two uncommitted edits that already existed before the audit: `crates/dc-core/MEMORY_LEAK_EVIDENCE.md` and `crates/dc-core/tests/extended_stability_soak.rs`. Their diff is saved. The new source SHA without this patch does not sufficiently describe the examined version.

To double-check the remark: source/binary/hash → desired API → negative example → actual exit/outcome → new solution. Static diagnosis and runtime reproduction are noted separately. If the implementation has changed, the new report gets a new baseline, and the old snapshot is not overwritten.

## Play

Process only MSVC environment; target/TEMP/TMP/AUDIT_TMP on D. Used original tools/Enter-BuildEnvironment.ps1. Don't run deployment scripts from this ZIP on the user's machine for easy reading: they install/uninstall/stop the application.

```powershell
cargo test --manifest-path <audit>/snapshot/Cargo.toml --locked --offline -p dc-domain -p dc-protocol -p dc-store -p dc-watchdog --features dc-protocol/test-peer-constructor,dc-store/sqlite
cargo test --manifest-path <audit>/probes/Cargo.toml --locked --offline
node node_modules/typescript/bin/tsc --noEmit -p tsconfig.json
node node_modules/vitest/vitest.mjs run
```

The UI checks for this pass were done from the original project; Rust from snapshot/probes. Default full workspace run did not run due to existing live test routes. Artifact probes use their saved Cargo.lock and do not require a real gateway. UI test failures are not hidden.

## Unverified

- Line-by-line semantic check of each historical file/chat; inventory - enumeration only.
- Full matrix of owner decisions and all C-01—C-16 with independently accepted evidence.
- Live Kanban and work cycle of all external agents; full attribution of their edits.
- Real window/clicks, full desktop build and compliance of all old installers with the current source SHA.
- Full soak of the new harness, power-off, loads of long-term jobs and all combinations of failures.
- Real provider calls, accounts, cookies/proxies, sandbox attacks, production pipe authentication.
- Full dependency vulnerability/license audit and proof of all migrations.

This pass did not fix the product code. New probes are part of the audit; changes are only allowed in the new report directory. Real repairs should be carried out in the candidate/worktree designated by the organization in accordance with current AGENTS.md/ORGANIZATION.md. Neighboring Draft, Hermes profile, keys and old archives do not change.

The quote about ha-otc-scanner busy files refers to a different project; it was not read or changed. A working SQLite needs a consistent backup, and not a blind copy of a busy db file. You cannot pass off someone else's copying errors as a Deep Create defect.
