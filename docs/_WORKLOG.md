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

## 2026-10-04 02:55 — management relay t_c6c87f42 (run251)
- Archived stale control/CONTINUATION.json (parent t_10e9e27a) -> control/archive/CONTINUATION_t_10e9e27a_stale_20261004.json
- Published canonical 4-field control/CONTINUATION.json (parent_task_id=t_c6c87f42, mode=waiting, next_actions=[])
- Appended PRODUCT_OBLIGATIONS.json checkpoint t_c6c87f42-20261004 (board_state_final: t_38823616 run252 LIVE C12 production-Drop injection, t_98ccb794 run250 LIVE C06 F1 reviewer, t_4bbdee15 blocked+owner-foreground-evidence-durable, t_8858a3c3 12/14 parents done, no budget resets, no PASS claimed)
- Appended control/HANDOFF.md cycle section
- Owner mid-run directive: t_69ad7ede run246 timed_out; owner reserved t_38823616 with same semantic locks (C02-authenticated-transport-source + C12-named-pipe-attestation-source); t_63f77ef8 additionally gated on it; old budget preserved; no duplicate writers
- changed files: control/CONTINUATION.json (sha256 1efc4c59...), control/archive/CONTINUATION_t_10e9e27a_stale_20261004.json (sha256 608da80a...), control/PRODUCT_OBLIGATIONS.json (sha256 d4edca6f...), control/HANDOFF.md (sha256 84f8cc46...)

## 2026-10-17 03:20 — Wave23: REJECT verdict processed, swarm graph live, for-work crash root-caused
- t_8538abb1 review DONE (1h26m, 148 msgs): verdict REJECT — wave21 patch targets phantom baseline 3755796c (absent in donor object store), storage drift (PersistedCoreState lacks PartialEq/Eq). REVIEW_VERDICT.json in wave22-review-20261016/.
- t_da936492 (merge) gated correctly — comment added, merge must NOT proceed; superseded by wave23 re-author.
- Created t_b320528a [wave23] re-author delta against f7444d1 (deepcreatekimi) — RUNNING.
- Read Conclusion.md MORE (line 742+): independent review 2026-10-01 — main P1 = N03 verifier accepts another unit's result as Passed; N08 schema 10->3 drift; 376 rust tests pass; UI 50/3; owner estimate 25-40%.
- Created t_d12bd8c1 [wave23] N03 verifier-integrity fix (strict TDD, isolated worktree).
- Created kanban SWARM t_b6de3fe5 (root) with workers t_32fe5363 (N08 schema fix), t_658291aa (IPC/execution-boundary), verifier t_d0b678a9 (deepcreatekimi), synthesizer t_73efa71c.
- ROOT CAUSE found: profile for-work = openai-codex/ChatGPT backend -> HTTP 400 on dash/* models -> agents crash-looped x2 on t_d12bd8c1 + t_658291aa, both auto-blocked. Fixed: reassign -> Deep Create contributors + set-model dash/deepseek-v4-pro-0813, unblocked, both RUNNING.
- Live now: t_ac1f4cbf (omniroute hygiene), t_b320528a (re-author), t_d12bd8c1 (N03), t_32fe5363 (N08), t_658291aa (IPC), t_1622b5e5 (P1 gaps) = 6 concurrent agents + queued verifier/synthesizer.

## 2026-10-17 03:45 — Wave23 swarm repaired + P1 gap plan converted to live kanban pipeline
- Swarm workers t_32fe5363/t_658291aa crashed x2: ROOT CAUSE = `hermes kanban swarm --worker PROFILE:TITLE:SKILLS` parsed my title text (commas) as skill names -> "Unknown skill(s)" spawn failure. Fix: archived broken workers, recreated as plain tasks t_de6f86d7 (N08 schema gate) + t_6cd1c7e9 (IPC/exec boundary), relinked verifier t_d0b678a9 -> new workers -> synthesizer t_73efa71c. LESSON: never put commas/skill-like text in swarm worker titles.
- t_1622b5e5 DONE (1h30m, 103 msgs): P1_GAP_PLAN.md written — facts per gap (A Windows-native, B Store Stage3, C crash recovery, D installer), 6 discrepancies incl. D-1: donor SHA 0c0bd0cf does NOT exist; real 0c0bd0c1... lives in ANOTHER repo; D-3: three evidence files cited as PROVEN (CRASH_RECOVERY/INSTALLER_VALIDATION/WEBVIEW_E2E) absent on disk; D-6: updater endpoint = placeholder.
- Converted plan tasks 1-4 into kanban cards directly from plan body-drafts: t_c43f2e41 (evidence integrity), t_d76a093d (native build wrapper), t_3b65af91 (real-kill crash recovery test, gated on build wrapper), t_2724f150 (Stage 3 DPAPI/encryption/crash-injection, gated on build wrapper).
- Board now: running=7 (t_ac1f4cbf hygiene, t_b320528a re-author, t_d12bd8c1 N03, t_de6f86d7 N08, t_6cd1c7e9 IPC, t_c43f2e41 evidence, t_d76a093d build), todo=7 gated queue (verifier+synthesizer, gap3, gap4, 3 stale qualified-execution), done=243.

## 2026-10-17 04:45 — Swarm capacity expanded: 2 new worker profiles + plan tasks 5/6/7 launched
- Created profiles deepcreateworker1/deepcreateworker2 (copies of deepcreatekimi config, dash/deepseek-v4-pro-0813) — spawn VERIFIED live (t_48d9fb5a running).
- Created remaining P1_GAP_PLAN tasks: t_48d9fb5a (gap7 independent review wave6->wave21 candidate chain, worker1), t_d8c1b901 (gap5 Windows isolation wiring Job Object/AppContainer, worker2, gated on gap4 t_2724f150), t_880e317b (gap6 installer NSIS/MSI + updater endpoint, kimi, gated on gap3 t_3b65af91).
- Plan execution order honored: 1||2||7 -> 3 -> 4 -> 5 -> 6. Board: running=8, todo=9 gated.

## 2026-10-17 04:25 — End-of-pipeline closed: integration + final acceptance tasks created
- Re-author t_b320528a PRODUCED OWNED_SOURCE_DELTA-v2.patch (wave23-rea-20261017/) — REJECT loop is closing.
- Created t_3f3f5a61 [wave23-integration]: merge verifier-approved outputs into wt-integration-20261017 off f7444d1, full-test gate (gated on synthesizer t_73efa71c + integration branch).
- Created t_3ef0308f [wave23-acceptance]: FINAL product acceptance -> RELEASE_READY_VERDICT.json + FULL_CORE_VERIFIED-v2.json (product_acceptance true only on READY). Gated on t_3f3f5a61 + t_b3533b8a.
- Re-pointed post-merge verify t_b3533b8a from cancelled t_da936492 to wt-integration-20261017 (comment + link).
- Full autonomous chain now: 8 running fixes -> verifier -> synthesizer -> integration -> post-merge verify -> final acceptance. Board: running=8, todo=11.

## 2026-10-17 06:55 — Pipeline tail corrected (cycle + stale SKIPPED gate found & fixed)
- DISCOVERY: t_b3533b8a (post-merge verify) had completed SKIPPED at 02:49 — it gated nothing real; naive link created a CYCLE (dispatch refused, good guard).
- Fix: created t_000a25e1 REAL post-merge verification of wt-integration-20261017 (gate failed==0, passed>=192, FULL_CORE_VERIFIED_POSTMERGE.json, honest-FAIL allowed).
- Final graph: 8 running fixes -> t_d0b678a9 verifier -> t_73efa71c synthesizer -> t_3f3f5a61 integration (wt-integration-20261017 off f7444d1) -> t_000a25e1 post-merge verify -> t_3ef0308f FINAL acceptance (RELEASE_READY_VERDICT.json, product_acceptance only on READY).
- Stale parent links pruned (t_da936492 cancelled, t_b3533b8a skipped). Board: running=8, todo=12 gated, done=244.

## 2026-10-17 07:40 — Timeout-crash wave repaired: 4 continuation tasks, zero work lost
- gap1 evidence integrity t_c43f2e41 DONE: CRASH_RECOVERY rebuilt with real TerminateProcess+WAL tests (exit 0 logs on disk), INSTALLER_VALIDATION honestly RETRACTED (MSI 1603 non-elevated). Phantom-PROVEN cleanup complete.
- ROOT ISSUE: 7200s max-runtime cap < real task duration (builds+tests) -> 4 agents timed out x2 and auto-blocked (t_b320528a, t_d12bd8c1, t_de6f86d7, t_6cd1c7e9).
- FIX: continuation tasks with verified on-disk state, max-runtime 21600, mandatory incremental checkpoint commits:
  t_42fb9d7a re-author finish (patch exists, TREE_MATCH=YES proven; only test run remains),
  t_f543454a N03 finish (worktree wave23/n03-verifier-integrity),
  t_23916d19 N08 finish (UNCOMMITTED work preserved: lib.rs/sqlite.rs mods + n08_repro.rs + schema_gate.rs tests),
  t_435bf72b IPC redo-fast (clean worktree; instructed targeted-crate builds only, no full-workspace rebuilds).
- Old blocked tasks archived; continuations linked to verifier t_d0b678a9. All 4 spawned RUNNING (verified). Board: running=7, todo=12, done=245.

## 2026-10-17 08:05 — Hygiene timeout repaired; gap2 build-wrapper effectively delivered
- t_ac1f4cbf (OmniRoute hygiene) timed out x2 at 10800s AFTER backup+research complete -> continuation t_40f69d72 APPLY-only (8 dead connects + dash fallback chain + smoke), old archived, new RUNNING (verified).
- gap2 t_d76a093d (build wrapper): REPORT.md + DC_BUILD_WRAPPER.patch (sha256 da2c0dae...) exist, git apply --check GREEN on f7444d1 — tools/dc-build.sh (161 lines) fixes the GNU link.exe shadowing class-failure permanently. Task finishing final steps.
- gap1 REPORT.md verified on disk: CRASH_RECOVERY rebuilt (2+1 tests passed, real TerminateProcess+WAL), INSTALLER_VALIDATION retracted honestly (MSI 1603), WebView2 E2E retracted. Phantom-PROVEN era closed.
- Board: running=7, todo=12 gated, done=244. Timeout-lesson applied: every continuation carries max-runtime 21600/10800 + checkpoint-commit discipline.
