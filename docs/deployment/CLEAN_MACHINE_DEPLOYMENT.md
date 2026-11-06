# Deep Create — Clean-Machine Deployment Guide

> Audience: an operator installing Deep Create on a **fresh Windows machine**
> with **no Rust, no cargo, no Node, no pnpm, no MSVC, no dev tools of any kind**.
> Everything below uses only the artifacts produced by the release build
> (`msi` / `nsis` bundles) plus stock Windows tooling.

---

## 1. Prerequisites (the ONLY allowed installs)

| Prerequisite | Why | How to get it on a clean machine |
|---|---|---|
| Windows 10 1809+ / Windows 11 (x64) | Tauri 2 runtime base | — |
| **Microsoft Edge WebView2 Runtime** | Tauri renders its UI with WebView2. The NSIS installer is configured to bootstrap it automatically; the MSI does **not**. | Preinstalled on Win 11 and up-to-date Win 10. Otherwise: <https://developer.microsoft.com/microsoft-edge/webview2/> → "Evergreen Standalone Installer" (x64). The NSIS installer can also fetch it during setup (requires network). |
| An account with permission to install software | MSI per-machine install requires elevation; NSIS defaults to per-user (no admin) | — |

**Nothing else is required.** No Visual C++ redistributable beyond what ships
with Windows 10/11 (the release binary is linked against the UCRT, which is
in-box since Windows 10). No Rust, no Node, no package managers.

Verify WebView2 presence (optional, PowerShell or cmd):

```cmd
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv
```

A `pv` value like `154.0.xxxx.xx` means the runtime is installed. An error
means it is missing.

---

## 2. Artifacts produced by the release build

Built from the repo with `pnpm exec tauri build` (build machine only — see
§6). Output location:

```
.local/target/release/bundle/
├── msi/Deep Create_<version>_x64_en-US.msi
└── nsis/Deep Create_<version>_x64-setup.exe
```

Copy **one** of these to the clean machine. NSIS is the recommended default
(per-user install, can auto-bootstrap WebView2, no admin needed).

---

## 3. Fresh-machine install — NSIS (recommended path)

1. Copy `Deep Create_<version>_x64-setup.exe` to the clean machine
   (e.g. `C:/home/researcher`).
2. Double-click the installer.
   - SmartScreen on a fresh machine will warn (unsigned binary) →
     "More info" → "Run anyway".
   - If WebView2 is missing, the installer offers to download it
     (network required) — accept.
3. Choose install scope. Default = current user only, installs to
   `%LOCALAPPDATA%\Programs\Deep Create`. No admin prompt.
4. Finish. The installer optionally launches the app; a Start-Menu shortcut
   **Deep Create** is created.

### Silent / scripted install (for VM automation)

```cmd
"Deep Create_0.1.0_x64-setup.exe" /S
```

`/S` = silent (case-sensitive NSIS flag). Per-machine silent install requires
an elevated shell and `/ALLUSERS=1` semantics are NSIS-build specific —
prefer per-user for unattended test rigs.

### Uninstall

- Settings → Apps → **Deep Create** → Uninstall, **or**
- run `%LOCALAPPDATA%\Programs\Deep Create\Uninstall.exe`.

---

## 4. Fresh-machine install — MSI (managed/enterprise path)

1. Copy `Deep Create_<version>_x64_en-US.msi` to the clean machine.
2. **Ensure WebView2 is already installed** (MSI will not bootstrap it —
   see §1). Without it the app installs but shows a blank window.
3. Install (elevated — MSI installs per-machine to `C:\Program Files\Deep Create`):

```cmd
msiexec /i "Deep Create_0.1.0_x64_en-US.msi" /qn /norestart /l*v install-msi.log
```

   `/qn` = fully silent, `/l*v` = verbose log (capture it as evidence).
   For interactive install just double-click the MSI.

4. Launch: Start Menu → **Deep Create**, or
   `"C:\Program Files\Deep Create\Deep Create.exe"`.

### Uninstall

```cmd
msiexec /x "Deep Create_0.1.0_x64_en-US.msi" /qn /norestart
```

---

## 5. First launch on a clean machine

1. Start **Deep Create** from the Start Menu.
2. The main window ("Deep Create - Build Information") appears with the
   version page. First launch may take a few seconds while WebView2
   initializes.
3. **Database**: the app creates/uses its SQLite store at the path given by
   the `DC_STORE_PATH` environment variable. On a clean machine with no env
   vars set, persistence is **disabled by design** (dev mode fallback) — see
   "Know limitation" below. To enable persistence for a production install,
   set a user env var once:

```cmd
setx DC_STORE_PATH "%LOCALAPPDATA%\DeepCreate\deepcreate.db"
```

   then relaunch. The app creates the file and all tables on first launch
   (no migrations to run by hand).

4. **Smoke check (GUI)**: the version page must render and show a version
   string and "work_graph" build stage. That proves: binary starts, WebView2
   renders, the `application_info` IPC command round-trips through the real
   invoke handler and ACL.

### Known limitation (honest note)

`DC_STORE_PATH` is currently read from the **process environment only**
(`main.rs:289`). There is no built-in default under `%APPDATA%` yet. On a
truly clean machine, either set the env var as above, or launch via the
validation script which sets it per-invocation. A follow-up should make the
app default to `dirs::data_local_dir()\DeepCreate\deepcreate.db`.

---

## 6. (Build machine only) How the bundles are produced

For completeness — this happens on the **dev machine**, never on the clean
target:

```bash
cd /d/Projects/DeepCreate
source .local/setup-build-env.sh   # MSVC/SDK/linker for the bundled toolchain
# Node >=22.12 <23 and pnpm 12.3.4 must be on PATH (engines field is enforced)
pnpm exec tauri build              # runs ui:build, cargo release build, bundlers
```

WiX (MSI) and NSIS toolchains are downloaded by `tauri build` on first use
into the user cache — they are build-time only and are **not** needed on the
target machine.

---

## 7. Automated validation

See `tools/Validate-CleanMachine.ps1` and `tools/validate-clean-machine.sh`
in this repo. They verify, in order:

1. Installer artifact exists and is well-formed
2. Install completes (or, for the isolated fallback: binary is staged to a
   clean directory)
3. App launches with a **sanitized environment** (no `CARGO_*`, `RUST*`,
   `LIB`, `INCLUDE`, dev `PATH` entries)
4. Database file + schema is created at `DC_STORE_PATH`
5. App responds to commands — verified two ways:
   - **Headless IPC harness** (exact command surface, MockRuntime-free):
     `cargo test -p dc-desktop` E2E tests already prove the invoke handler;
     for the installed binary, the GUI version page rendering proves
     `application_info` round-trip.
   - **Persistence probe**: relaunch and confirm the DB is reopened and
     non-empty (state recovered — see stderr log lines `Recovered persisted
     state` / `Objective restored` etc.)
6. Uninstall leaves no residue (install dir removed; DB dir untouched by
   uninstaller — user data is preserved by design).

Evidence from the latest run is collected in `DEPLOYMENT_EVIDENCE.md` at the
repo root.
