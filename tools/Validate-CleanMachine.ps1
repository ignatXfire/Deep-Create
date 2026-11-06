# Validate-CleanMachine.ps1 — deployment validation for a FRESH Windows VM.
#
# Run this ON the clean machine (Windows 10/11 x64, WebView2 runtime, no dev
# tools). Copy this script plus the installer artifact onto the VM first.
#
# Checks, in order:
#   1. Installer artifact exists
#   2. Silent install succeeds (NSIS per-user, or MSI per-machine)
#   3. App launches and stays alive
#   4. Database is created at %LOCALAPPDATA%\DeepCreate\deepcreate.db
#      (script sets DC_STORE_PATH for the launched process only)
#   5. Relaunch re-opens the DB (persistence recovery)
#   6. Uninstall removes install dir, preserves DB
#
# Usage (elevated not required for NSIS per-user):
#   powershell -ExecutionPolicy Bypass -File Validate-CleanMachine.ps1 `
#       -Installer "C:/home/researcher Create_0.1.0_x64-setup.exe"
#
#   For MSI:
#   powershell -ExecutionPolicy Bypass -File Validate-CleanMachine.ps1 `
#       -Installer "C:/home/researcher Create_0.1.0_x64_en-US.msi" -Msi

param(
  [Parameter(Mandatory=$true)][string]$Installer,
  [switch]$Msi,
  [string]$WorkDir = "$env:TEMP\dc-validate-$(Get-Date -Format yyyyMMdd-HHmmss)"
)

$ErrorActionPreference = 'Stop'
$log = Join-Path $WorkDir 'validate.log'
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$script:failed = $false

function Pass($m) { "PASS: $m" | Tee-Object -Append -FilePath $log }
function Fail($m) { "FAIL: $m" | Tee-Object -Append -FilePath $log; $script:failed = $true; throw $m }

"=== Deep Create clean-machine validation $(Get-Date -Format o)" | Out-File $log
"host: $env:COMPUTERNAME / $env:USERNAME" | Out-File -Append $log
"os:  $([System.Environment]::OSVersion.VersionString)" | Out-File -Append $log

# --- 1. artifact -------------------------------------------------------------
if (-not (Test-Path $Installer)) { Fail "installer not found: $Installer" }
Pass "installer exists: $Installer ($([math]::Round((Get-Item $Installer).Length/1MB,1)) MB)"

# --- 2. WebView2 prerequisite -------------------------------------------------
$wv = Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
if (-not $wv) { $wv = Get-ItemProperty 'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue }
if ($wv -and $wv.pv) { Pass "WebView2 runtime present: $($wv.pv)" } else { Fail "WebView2 runtime missing" }

# --- 3. install ---------------------------------------------------------------
if ($Msi) {
  $msiLog = Join-Path $WorkDir 'msi-install.log'
  $p = Start-Process msiexec.exe -ArgumentList '/i', "`"$Installer`"", '/qn', '/norestart', '/l*v', "`"$msiLog`"" -Wait -PassThru
  if ($p.ExitCode -ne 0) { Fail "msiexec exit $($p.ExitCode) — see $msiLog" }
  $exe = "C:\Program Files\Deep Create\Deep Create.exe"
  Pass "MSI install completed"
} else {
  $p = Start-Process $Installer -ArgumentList '/S' -Wait -PassThru
  if ($p.ExitCode -ne 0) { Fail "NSIS installer exit $($p.ExitCode)" }
  $exe = Join-Path $env:LOCALAPPDATA 'Programs\Deep Create\Deep Create.exe'
  Pass "NSIS silent install completed"
}
if (-not (Test-Path $exe)) { Fail "installed binary not found at expected path: $exe" }
Pass "installed binary present: $exe"

# --- 4. launch with clean DB path ----------------------------------------------
$dbDir = Join-Path $WorkDir 'db'
New-Item -ItemType Directory -Force -Path $dbDir | Out-Null
$db = Join-Path $dbDir 'deepcreate.db'

$proc = Start-Process $exe -PassThru -Environment @{ DC_STORE_PATH = $db } 2>$null
if (-not $proc) {
  # Start-Process -Environment needs PowerShell 7; fall back to cmd /c set
  $proc = Start-Process cmd.exe -ArgumentList '/c', "set DC_STORE_PATH=$db&& `"$exe`"" -PassThru -WindowStyle Hidden
}
Start-Sleep -Seconds 8
$alive = Get-Process -Id $proc.Id -ErrorAction SilentlyContinue
if (-not $alive) {
  $alive = Get-Process | Where-Object { $_.Path -eq $exe } | Select-Object -First 1
}
if (-not $alive) { Fail "app did not start or exited immediately" }
Pass "app launched (pid $($alive.Id)) and alive after 8s"

# --- 5. database created --------------------------------------------------------
if (-not (Test-Path $db)) { Fail "database not created at $db" }
$len = (Get-Item $db).Length
if ($len -le 0) { Fail "database file empty" }
Pass "database created: $db ($len bytes)"

# --- 6. stop + relaunch (persistence recovery) -----------------------------------
$alive | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process | Where-Object { $_.Path -eq $exe } | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
$proc2 = Start-Process cmd.exe -ArgumentList '/c', "set DC_STORE_PATH=$db&& `"$exe`"" -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 6
$alive2 = Get-Process -Id $proc2.Id -ErrorAction SilentlyContinue
if (-not $alive2) { $alive2 = Get-Process | Where-Object { $_.Path -eq $exe } | Select-Object -First 1 }
if (-not $alive2) { Fail "app failed to relaunch against existing DB" }
Pass "relaunch against existing DB OK (persistence re-init)"
$alive2 | Stop-Process -Force -ErrorAction SilentlyContinue

# --- 7. uninstall -------------------------------------------------------------------
if ($Msi) {
  $p = Start-Process msiexec.exe -ArgumentList '/x', "`"$Installer`"", '/qn', '/norestart' -Wait -PassThru
  if ($p.ExitCode -ne 0) { Fail "MSI uninstall exit $($p.ExitCode)" }
} else {
  $un = Join-Path (Split-Path $exe) 'Uninstall.exe'
  if (Test-Path $un) { Start-Process $un -ArgumentList '/S' -Wait } else { "WARN: uninstaller not found at $un" | Tee-Object -Append -FilePath $log }
}
Start-Sleep -Seconds 3
if (Test-Path (Split-Path $exe)) { Fail "install dir still present after uninstall: $(Split-Path $exe)" }
Pass "uninstall removed install dir"
if (Test-Path $db) { Pass "user data (DB) preserved after uninstall — by design" }

"" | Out-File -Append $log
if ($script:failed) { "RESULT: FAILED — see $log" ; exit 1 } else { "RESULT: ALL CHECKS PASSED — log: $log" }
