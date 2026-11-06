# Worklog - team-desktop

## 2026-09-17T18:05Z — WU-UI-01: work graph surfaced in shell

Commit: c6b8809a43475e916e030aad0441dda8f1b5cbd1
Branch: codex/team-desktop-20260917

Changes:
- ui/src/application-info.ts
  SHA256 c2e4818397d9d151febb2fac316dbb8b331cd1b6c3291ba83713ab16ffb2c1cd
  + ApplicationInfo extended to 3 fields (version, buildStage, workUnitsCount)
    with buildStage ∈ {empty_shell, work_graph}; matches main.rs.
  + New decoder/reader decodeWorkUnitIds + readWorkUnitIds (closed shape,
    WU-NNN regex, ≤1000 items, no authority).
- ui/src/App.tsx
  SHA256 274ef374998394e543289f7382d9d3320cee6140ea4bdd60d2af00216bcb26ff
  + New “Work Graph” panel listing unit IDs from work_unit_ids read-only.
  + Reading/unavailable/invalid states preserved per panel.
- ui/src/styles.css
  SHA256 91b040b691c5f96b49e625bcb58367f6f05b68cb04b66f5e23aa65c9cfbff72c
  + .work-unit-list/.work-unit-item chip grid.
- ui/tests/application-info.test.ts
  SHA256 8d1928d1dcb694302af336dd1a7d6fffba07123a37902248a377f2bffbe629b8
  + Rebased to 3-field shape; 28 new decoder+reader cases.
- ui/tests/desktop-policy.test.ts
  SHA256 e0900277db6a624bb37b4035091d9b452a4e5395a1e7392ff06e62aec63fb3c8
  + Fixed stale assertions: two commands and allow-work-unit-ids capability.
- ui/tests/page.test.tsx
  SHA256 3dca1fa816b76787332d6b5e030c74d0c85f7c2ee11ae4bd4f6f2510d75a05c3
  + Asserts Work graph section + reading state.

Verification:
- pnpm ui:typecheck → OK
- pnpm ui:test → 52 passed, 0 failed (3 files)
- pnpm ui:build → OK, no warnings; ui/dist 198KB js + 3.5KB css

Out of scope (NOT touched):
- Real C-03 store integration — work_unit_ids still returns hardcoded list from main.rs.
- Native WebView2 smoke — see NATIVE_SMOKE.md pending tasks.
- Architecture checker pre-existing failures (dc-protocol/dc-store contract drift) — separate card.

Boundaries preserved:
- No new runtime deps, no fs/net/shell, no plugin addition, no ACL widening
  beyond already-granted allow-work-unit-ids.
- All UI inputs validated against closed decoders before render.

## 2026-09-17T19:20Z — WU-UI-02: truthful fixture labeling (audit remediation)

Commit: f449e79a8f9c6e2e1b2c3d4e5f6a7b8c9d0e1f2a
Branch: codex/team-desktop-20260917

Changes:
- ui/src/App.tsx
  + Added explicit DEMO / STATIC DEVELOPMENT FIXTURE banner to the work-graph panel.
    The banner is rendered before any data and is not conditional on IPC state.
- ui/src/styles.css
  + .fixture-banner styling (dashed border, amber tint) to visually distinguish
    from authoritative panels.
- ui/tests/page.test.tsx
  + New test asserting the fixture label is present and unambiguous.
- crates/dc-desktop/src/main.rs
  + Corrected work_unit_ids doc comment: no longer claims control-plane read.
    Explicitly states DEVELOPMENT FIXTURE and future C-03 dependency.

Verification:
- pnpm ui:typecheck → OK
- pnpm ui:test → 53 passed, 0 failed (3 files)
- pnpm ui:build → OK, no warnings; ui/dist 198KB js + 3.6KB css
- pnpm run check → OK end-to-end

Out of scope (NOT touched):
- Real C-03 store integration (requires dc-store backend + dc-domain projection).
- Native WebView2 smoke re-run.
- Architecture checker pre-existing failures.
