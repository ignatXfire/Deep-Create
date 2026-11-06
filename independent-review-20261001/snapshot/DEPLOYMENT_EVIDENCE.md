# DEPLOYMENT EVIDENCE — Deep Create clean-machine validation

**Date:** 2026-10-01 (Europe/Berlin)
**Commit:** `bdba109` (working tree has unrelated pre-existing modifications)
**Host:** owner-specific environment metadata omitted for publication; this remains a historical report.
**VM availability:** No fresh Windows VM/container available on this host → per task instructions, the most isolated on-machine test was performed (§3).

---

## 1. Release artifacts built from scratch

`pnpm exec tauri build` (full release rebuild, 31m 36s, exit 0). This is the
first successful bundle build in the project's history — previous attempts
(`tauri-bundle-build-3.log`) failed because the wrong Node runtime (v24,
violating `engines: >=22.12 <23`) was on PATH; using the fnm-managed
Node v22.22.3 + pnpm 12.3.4 fixed it.

| Artifact | Size | SHA-256 |
|---|---|---|
| `.local/target/release/dc-desktop.exe` | 12 MB | `d81517b5ab…5bc65` |
| `bundle/nsis/Deep Create_0.1.0_x64-setup.exe` | 3.5 MB | `83df2a2d17…d7851` |
| `bundle/msi/Deep Create_0.1.0_x64_en-US.msi` | 4.9 MB | `609c591301…39429` |

Build log: `.local/deploy-validation-build.log` (ends `EXIT=0`, 2 bundles finished).
Compiler warnings only (3 unused imports) — no errors.

## 2. Fresh-install procedure documented

`docs/deployment/CLEAN_MACHINE_DEPLOYMENT.md` — exact steps for a machine
with no Rust/cargo/Node/MSVC: prerequisites (WebView2 only), NSIS and MSI
install paths (interactive + silent commands), first-launch behaviour,
`DC_STORE_PATH` setup, uninstall, and the honest known limitation that the
app currently has no built-in default DB path (reads `DC_STORE_PATH` env var
only — follow-up noted).

Validation tooling:
- `tools/Validate-CleanMachine.ps1` — for a real fresh VM (install → launch →
  DB → relaunch → uninstall, all logged).
- `tools/validate-clean-machine.sh` — isolated on-machine validation
  (staging dir, sanitized `env -i` environment, fresh DB path).

## 3. Isolated on-machine validation (VM substitute) — PASSED

`tools/validate-clean-machine.sh` run log
(`/tmp/dc-deploy-20261001-045632/run.log`):

```
PASS: release binary found: .local/target/release/dc-desktop.exe
PASS: staged clean install dir at /tmp/dc-deploy-…/stage (12M)
PASS: WebView2 runtime present (154.0.4258.37)
PASS: app launched under sanitized env (pid 3085) and is alive after 6s
PASS: database created at …/db/deepcreate.db (4096 bytes)
PASS: DB has SQLite header
PASS: DB contains dc-store schema tables (entities/store_meta) [db+wal]
PASS: uninstall residue check: staging dir removed, DB preserved
== DONE: all checks passed ==
```

Isolation properties: binary copied to a fresh dir; process started with
`env -i` (PATH = only `C:\Windows\System32;C:\Windows` — no cargo, no MSVC
`LIB`/`INCLUDE`, no `CARGO_*`/`RUST*`, no dev tools); `DC_STORE_PATH` pointed
at a brand-new empty directory. Schema is committed to the WAL
(`journal_mode=WAL`, `synchronous=FULL` per dc-store), so validation greps
both `.db` and `.db-wal`.

## 4. Real NSIS installer end-to-end — PASSED

- Silent install: `Deep Create_0.1.0_x64-setup.exe /S` → exit 0.
- Registered in `HKCU\…\Uninstall\Deep Create`, installed to
  `C:/home/researcher Create\dc-desktop.exe` (per-user, no
  admin required).
- Launched the **installed** binary under a sanitized environment with a
  fresh DB path → process alive, `installed.db` + `-wal` (412 KB) created,
  schema markers present in WAL.
- `uninstall.exe /S` → exit 0, install directory fully removed.

## 5. Command-surface (responds to commands) — PASSED

`cargo test -p dc-desktop --bin dc-desktop` — drives `application_info`,
`work_unit_ids`, `work_units`, `submit_objective`, `plan_objective_graph`,
`accept_graph`, `health_check` through the **real Tauri invoke handler**
(IPC deserializer + ACL + command router), plus ACL denial tests:

```
test result: ok. 10 passed; 0 failed; 1 ignored (live broker dispatch)
```

The installed binary's GUI version page exercises `application_info` on
every launch, which is covered by §4's successful run.

## 6. Known limitations / follow-ups (honest)

1. **MSI silent install fails non-elevated** with msiexec exit 67 → 1603
   (log: `.local/.../msi-install.log`). Per-machine MSI requires an elevated
   shell — standard Windows behaviour, not a defect; the NSIS per-user path
   (recommended) needs no elevation. Elevated MSI install not tested here
   (would modify this machine's `C:\Program Files`).
2. **No default DB path**: without `DC_STORE_PATH` the app runs with
   persistence disabled (dev-mode fallback). Follow-up: default to
   `%LOCALAPPDATA%\DeepCreate\deepcreate.db`.
3. **GUI rendering not screenshot-verified** headlessly: window creation is
   proven by process liveness + WebView2 presence; visual check remains a
   manual step on a real fresh machine.
4. **True fresh-VM run not performed** (none available): the PS1 script is
   ready for it; the sanitized-env run reproduces the key conditions
   (no dev tools on PATH, no env vars, fresh user-data dir).

## Verdict

Clean-machine deployment is **validated to the extent possible without a
fresh VM**: bundles build, NSIS installs/launches/creates DB/uninstalls
cleanly on a sanitized environment, command surface responds correctly, and
the exact fresh-machine procedure is documented with a ready-to-run VM
validation script.
