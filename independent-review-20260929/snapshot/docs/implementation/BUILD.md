# Local build Deep Create

Workplace: `D:/Projects/DeepCreate`. The document describes the local framework used and a reproducible helper for its Microsoft components. This is not a proof of build on a clean machine, an application installer, or a statement of complete system readiness.

## Dependencies of this environment

- Rust `1.92.0`, MSVC target `x86_64-pc-windows-msvc`; the version is fixed in `rust-toolchain.toml`, `rustfmt` and `clippy` are needed.
- Node.js `22.22.2`; root `package.json` allows the range `>=22.12.0 <23`, but the exact version specified was used here.
- pnpm `12.3.4`, assigned to `package.json`; packages are recorded by `pnpm-lock.yaml`.
- MSVC cache packages `Microsoft.VC.14.44.17.14`, extracted inside `.local/toolchain/msvc`. The actual tool path inside VSIX is `VC/Tools/MSVC/14.44.35207`, and the cache package versions are `14.44.35222`/`14.44.35223`. These are different designations, not a typo.
- Windows SDK cache `Win11SDK_10.0.26100,version=10.0.26100.15,productarch=neutral`; actual include/lib/bin revision is `10.0.26100.0`.
- Windows PowerShell 5.1 or PowerShell 7 for helper; Windows Installer and Authenticode verification are required for the SDK. Launching a native Tauri window also depends on the WebView2 Runtime installed; this helper doesn't install it.

Rust, Node and npm should already be in PATH. Global pnpm is not required: the tested commands below use a pinned local copy via `npm exec`. The Microsoft Components provisioning script does not install them or edit the user profile. `Enter-BuildEnvironment.ps1` adds only the extracted MSVC/SDK to the current PowerShell process and directs the TEMP/TMP/npm cache to the project's `.local`. General `Verify.ps1` requires PowerShell 7; The preparation script itself is also designed for Windows PowerShell 5.1.

## Preparation from existing cache

Stop building this checkout first: SDK preparation should not occur simultaneously with linking or compilation. `-ValidateOnly` does not perform MSI, unpack packages, or generate a provisioning report.

```powershell
Set-Location D:/Projects/DeepCreate
.\tools\Prepare-CachedWindowsToolchain.ps1 -ValidateOnly
```

If the inputs are checked, explicit preparation:

```powershell
.\tools\Prepare-CachedWindowsToolchain.ps1
. .\tools\Enter-BuildEnvironment.ps1
```

By default, only available inputs are used. The source of the four VSIX is the exact directories in `C:\ProgramData\Microsoft\VisualStudio\Packages`; SHA256 are assigned to helper:

| Cache payload | SHA256 |
|---|---|
| `Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base,version=14.44.35223,chip=x64/payload.vsix` | `CC1C0D0F44F17EC28CF9F9755A40AD6C5B73B35538ED593CB931CFCE5D32D2D3` |
| `Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.Res.base,version=14.44.35223,chip=x64,language=ru-RU/payload.vsix` | `D87D38B407734385FA879C174D844DCFC3E7E89275EBC8341FF0EB0181A3E815` |
| `Microsoft.VC.14.44.17.14.CRT.x64.Desktop.base,version=14.44.35222/payload.vsix` | `3D6B398F32EAFD7FB3735F83402C0658D831C3B4EE10F93F0031F7A78F311C27` |
| `Microsoft.VC.14.44.17.14.CRT.x64.Store.base,version=14.44.35222/payload.vsix` | `2FEA29C858FC487873E813926D240C97235153C2BEF82BF22D17CCE45E7BCF57` |

The fifth payload is `.local/toolchain/crt-headers.vsix`, SHA256 `852382A9AA73502B7849C1BCADFB603BA7175C4E8B60E6ABA03C7DE711D4ECE5`. It has already been obtained from [exact official Microsoft payload](https://download.visualstudio.microsoft.com/download/pr/c610cd8c-801b-44b8-a80a-82cc382aeb43/852382a9aa73502b7849c1bcadfb603ba7175c4e8b60e6aba03c7de711d4ece5/Microsoft.VC.14.44.17.14.CRT.Headers.base.vsix). Only if this file does not exist can its acquisition be explicitly allowed:

```powershell
.\tools\Prepare-CachedWindowsToolchain.ps1 -AllowDownload
```

Other versions, mirrors and automatic redirects are not used. An already existing file with an incorrect hash is not replaced by the download. `-AllowDownload` does not download missing VSIX/SDK cache packages. When transferring a trusted cache, you can set `-CacheRoot <путь>`; The names of the four packages, SHA256 and SDK edition remain fixed. `-ValidateOnly` and `-AllowDownload` cannot be mixed.

The SDK is extracted from the nine exact MSIs inside `Win11SDK_10.0.26100,version=10.0.26100.15,productarch=neutral/Installers`:

1. `Windows SDK Desktop Libs x64-x86_en-us.msi`
2. `Universal CRT Headers Libraries and Sources-x86_en-us.msi`
3. `Windows SDK Desktop Tools x64-x86_en-us.msi`
4. `Windows SDK for Windows Store Apps Libs-x86_en-us.msi`
5. `Windows SDK for Windows Store Apps Headers-x86_en-us.msi`
6.`Windows SDK for Windows Store Apps Headers OnecoreUap-x86_en-us.msi`
7. `Windows SDK Desktop Headers x86-x86_en-us.msi`
8. `Windows SDK Desktop Headers x64-x86_en-us.msi`
9. `Windows SDK for Windows Store Apps Tools-x86_en-us.msi`

Before changing outputs, helper checks the existence of inputs, assigned VSIX hashes, `Valid` Microsoft Authenticode signatures of MSI and the presence of external CABs from the read-only table `Media`. CAB and MSI hashes are included in the report. Signature verification uses Windows trust and can access the certificate verification infrastructure; banning new payload downloads is not a promise to shut down the entire OS network.

Only `Contents/VC/Tools/MSVC/14.44.35207/*` is read from VSIX. `..`, ambiguous segments, ADS/device names, link-like entries and leaving the project are prohibited; existing reparse-point ancestors, unpacking size and conflicting payloads are checked. Matching files are skipped; the modified file is replaced only by a ready-made, verified temporary file. Extraneous files are not deleted. This is helper's protection against dangerous path contents, not a guarantee against an arbitrary fully trusted process simultaneously hijacking the file system.

The MSI runs hidden via administrative extraction:

```text
msiexec.exe /a "<точный MSI>" /qn TARGETDIR="<checkout>\.local\toolchain\sdk" /norestart /L*v "<местный лог>"
```

This is `/a`, not the normal `/i` product installation. Helper does not require changing global profiles and does not involve rebooting. The output of the components is directed only to the current checkout on D; Windows Installer service behavior does not pass off as proven OS isolation. MSI error stops preparation without successful report; possible partial files and the diagnostic log remain for parsing.

If successful, the necessary compiler/header/lib/SDK files are checked and a new `.local/toolchain/preparation-reports/preparation-<20-значный номер>.json` is saved. The numbers are monotonous, past reports are not overwritten, the parallel helper is limited by file locking. The report contains sources, hashes, MSI signatures and control output hashes. This is a local provisioning log, not a cryptographic authority anchor, a complete installation record, or proof of power-down safety.

## Verification and build commands

The following commands correspond to pinned workspace/package scripts. Installing JS dependencies may require networking and is done separately from Microsoft-helper:

```powershell
Set-Location D:/Projects/DeepCreate
. .\tools\Enter-BuildEnvironment.ps1
rustc --version
node --version
npm exec --yes --package=pnpm@12.3.4 -- pnpm --version
npm exec --yes --package=pnpm@12.3.4 -- pnpm install --frozen-lockfile
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm exec --offline --yes --package=pnpm@12.3.4 -- pnpm check
node tools/check-architecture.mjs
node --test tools/tests/check-architecture.test.mjs
cargo build -p dc-desktop --locked --offline
```

It was the executable file `target/debug/dc-desktop.exe` that was checked after assembling the interface and `cargo build`. The `npm exec --yes --package=pnpm@12.3.4 -- pnpm desktop:build` command additionally calls `tauri build --no-bundle` for the release assembly; this path has not yet been verified and is not signed by the installer. `.local` and `target` contain ignored work products, not original contracts. Total repeatable pass: `.\tools\Verify.ps1`; his successful report is the main source of facts about a particular editorial office.

## What's checked now

The script is prepared on top of an already manually extracted and used build environment. After correcting the syntax error found during parsing, `-ValidateOnly` was actually completed: **5 VSIX, 9 signed MSIs, 38 external CABs, 512 scheduled files; no errors**. No extraction or MSI actions were performed during this scan. The new entire scenario with repeated `/a` has not yet been tested; the actual assembly used components previously extracted in the same way. General `Verify.ps1` now checks the syntax of all PowerShell scripts and this pre-pass so that repeating such an error will stop the check.

The dependency pinned rebuild is tested against the existing local environment and uses its intermediate results. This is not proof of identical bytes after a complete build from scratch on another computer. The actual results of the entire assembly and testing are recorded by `docs/implementation/STATUS.md`, and not by the fact of the existence of this instruction.

Not tested: clean machine without cache, automatic receipt of all Microsoft packages, full installer, SDK provisioning in all versions of Windows, power off, protected runtime/broker/watchdog, full application ready. A missing cache is an honest refusal with the exact file, and not a reason to quietly install the system Visual Studio.
