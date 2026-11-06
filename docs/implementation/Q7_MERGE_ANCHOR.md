# Q7 evidence anchor — team-context cc75170 merged into candidate

Run: kanban t_50ac2236 (deepcreatekimi, run 75).
Date: 2026-09-19 UTC.

## Merge

- Branch merged: `codex/team-context-20260917` at commit `cc75170` (reviewed PASS t_106b928d).
- Merge strategy: `git merge --no-ff cc75170` → clean, no conflicts (ort).
- Merged HEAD: `0cd0637880cda08e9cf8f8e182468e20db5dd140`
- Pre-merge candidate HEAD: `594c239b9ecd31d3f96a2b11f09e87ad2bb3b2ff` (Q6 scheduler merge).

## Files added/changed (594c239..0cd0637)

- `crates/dc-domain/src/context.rs`           +581 (new)
- `crates/dc-domain/src/lib.rs`                 +1  (`pub mod context;`)
- `crates/dc-domain/src/self_development.rs`   64  (rustfmt-only)
- `crates/dc-domain/src/work/explanation.rs`   16  (rustfmt-only)
- `crates/dc-domain/src/work/goal_change.rs`   20  (rustfmt-only)
- `tools/env-worktree.sh`                       +34  (new; worktree-local env helper)

SHA256 (post-merge):
- `crates/dc-domain/src/context.rs` = `83a40b46272c0dedc975d8368f0edce8363ecf2f774d0d7dcfb5dbaf13c5b7be`
- `tools/env-worktree.sh`           = `e9db6562adaa42fcb597a4852aa911c27cd599a4c4977bc71c66f7154586afb1`

## Verification (post-merge, worktree-local target dir)

Note F3 mitigation: candidate `.cargo/config.toml` pins `build.target-dir` to a
shared org-level path. To force a worktree-local target we overrode with
`CARGO_TARGET_DIR='D:/Projects/DeepCreate\.local\hermes-organization-20260916\candidate\target-integrate-q7'`
(Windows-style absolute path; MSYS form confuses `link.exe`).

Command: `source ../build-env.sh && export CARGO_TARGET_DIR=<above> && cargo test --workspace --no-fail-fast`
Exit code: 0.

Per-crate test counts (unit + integration + doc):

- `dc_desktop`
  - `crates/dc-desktop/src/main.rs` unit tests: 6
- `dc_domain`
  - `crates/dc-domain/src/lib.rs` unit tests: 33
  - `crates/dc-domain/tests/authority_resources.rs`: 29
  - `crates/dc-domain/tests/identity_contracts.rs`: 2
  - `crates/dc-domain/tests/lifecycle_contracts.rs`: 51
  - `crates/dc-domain/tests/scheduling_contracts.rs`: 19
  - `crates/dc-domain/tests/usage_bindings.rs`: 8
  - `crates/dc-domain/tests/work_contracts.rs`: 50
  - `crates/dc-domain` doc-tests: 13
  - Subtotal: 205
- `dc_protocol`
  - `crates/dc-protocol/src/lib.rs` unit tests: 5
  - `crates/dc-protocol/tests/authority.rs`: 0
  - `crates/dc-protocol/tests/envelopes.rs`: 9
  - `crates/dc-protocol/tests/outcomes.rs`: 8
  - `crates/dc-protocol` doc-tests: 0
  - Subtotal: 22
- `dc_store`
  - `crates/dc-store/src/lib.rs` unit tests: 8
  - `crates/dc-store/tests/integration_domain.rs`: 4
  - `crates/dc-store` doc-tests: 0
  - Subtotal: 12

Workspace total: 6 + 205 + 22 + 12 = **245 passed / 0 failed**
Expected: 232 (HEAD 594c239 anchor) + 13 (cc75170 context tests) = 245. MATCH.

`dc_domain` subtotal expected 192 + 13 = 205. MATCH.

## pnpm run check

Command: `pnpm run check` from candidate root.
Exit code: 0.
- `tsc --noEmit` (ui:typecheck) clean.
- `vitest run` — 3 test files / 53 tests, all pass.
- `vite build` — production bundle built (19 modules transformed, 305ms).

## Next serial merge

C-16 lineage (`0cf9ed3+c7a2b17`, `dc-execution`) is under fresh review in
t_df71218f (Deep Create contributors). When it PASSes, the next serial integrator
card merges it. One merge per run; serial only.
