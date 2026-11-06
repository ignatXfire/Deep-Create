#Requires -Version 7.0
# Executable evidence for this snapshot; not certification of the entire system.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$run = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')
$logRoot = Join-Path $root ".local\verification\$run"
$reportRoot = Join-Path $root 'docs\implementation\checks'
$null = New-Item -ItemType Directory -Path $logRoot, $reportRoot -Force
$steps = [Collections.Generic.List[object]]::new()
$report = [ordered]@{ run = $run; complete = $false; boundary = 'Current implemented slice only; not full application acceptance or power-loss proof'; steps = $steps; artifacts = @() }
$failure = $null

function Invoke-Check {
    param([string]$Name, [string]$Executable, [string[]]$Arguments)
    $log = Join-Path $logRoot "$Name.log"
    $started = [DateTime]::UtcNow
    & $Executable @Arguments *> $log
    $code = $LASTEXITCODE
    $steps.Add([ordered]@{
        name = $Name; executable = $Executable; arguments = $Arguments
        started_utc = $started.ToString('o'); ended_utc = [DateTime]::UtcNow.ToString('o')
        exit_code = $code; log = [IO.Path]::GetRelativePath($root, $log)
    })
    if ($code -ne 0) {
        Get-Content -LiteralPath $log -Tail 45 | Write-Host
        throw "Check $Name exited with code $code"
    }
    Write-Host "PASS $Name"
}

Push-Location $root
try {
    $powerShell = Join-Path $PSHOME 'pwsh.exe'
    Invoke-Check 'powershell-syntax' $powerShell @('-NoProfile', '-Command', '$failed=$false; Get-ChildItem -LiteralPath tools -Filter *.ps1 -File | ForEach-Object { $t=$null; $e=$null; [void][System.Management.Automation.Language.Parser]::ParseFile($_.FullName,[ref]$t,[ref]$e); if($e.Count){ $failed=$true; $e | ForEach-Object Message } }; if($failed){exit 1}')
    Invoke-Check 'toolchain-preflight' $powerShell @('-NoProfile', '-File', 'tools/Prepare-CachedWindowsToolchain.ps1', '-ValidateOnly')
    Invoke-Check 'contract-baseline' $powerShell @('-NoProfile', '-Command', '$r=Get-Content -LiteralPath docs/technical/checks/design_check_20260908.json -Raw | ConvertFrom-Json; foreach($f in $r.input_files){ if([IO.Path]::GetFileName($f.name) -ne $f.name){throw "Invalid baseline path"}; if((Get-FileHash -LiteralPath (Join-Path docs/technical $f.name) -Algorithm SHA256).Hash -ne $f.sha256){throw "Baseline differs: $($f.name)"} }; Write-Output "Verified $($r.input_files.Count) baseline documents"')
    . (Join-Path $PSScriptRoot 'Enter-BuildEnvironment.ps1')
    Invoke-Check 'rust-format' 'cargo' @('fmt', '--all', '--check')
    Invoke-Check 'architecture-selftests' 'node' @('--test', 'tools/tests/check-architecture.test.mjs')
    Invoke-Check 'architecture' 'node' @('tools/check-architecture.mjs')
    Invoke-Check 'contract-map-selftests' 'node' @('--test', 'tools/tests/check-contract-map.test.mjs')
    Invoke-Check 'contract-map' 'node' @('tools/check-contract-map.mjs')
    Invoke-Check 'topology-storage-contract' 'node' @('--test', 'tools/tests/topology-storage-contract.test.mjs')
    Invoke-Check 'external-request-profiles' 'node' @('--test', 'tools/tests/external-request.test.mjs')
    Invoke-Check 'rust-tests' 'cargo' @('test', '--workspace', '--locked', '--offline')
    Invoke-Check 'rust-lints' 'cargo' @('clippy', '--workspace', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings')
    Invoke-Check 'ui-check' 'npm.cmd' @('exec', '--offline', '--yes', '--package=pnpm@12.3.4', '--', 'pnpm', 'check')
    Invoke-Check 'desktop-build' 'cargo' @('build', '-p', 'dc-desktop', '--locked', '--offline')
    Invoke-Check 'desktop-repeat-build' 'cargo' @('build', '-p', 'dc-desktop', '--locked', '--offline')
    $files = @('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml', 'package.json', 'pnpm-lock.yaml', 'pnpm-workspace.yaml', 'tsconfig.json', 'vite.config.ts', 'target/debug/dc-desktop.exe', 'docs/technical/checks/design_check_20260908.json')
    $baseline = Get-Content -LiteralPath 'docs/technical/checks/design_check_20260908.json' -Raw | ConvertFrom-Json
    $files += @($baseline.input_files | ForEach-Object { Join-Path 'docs/technical' $_.name })
    $files += @(rg --files crates ui tools | Where-Object { $_ -notmatch '(^|[\\/])(gen|dist)[\\/]' })
    if ($LASTEXITCODE -ne 0) { throw 'Could not obtain the checked-file inventory' }
    $files += @(rg --files docs/implementation/contracts)
    if ($LASTEXITCODE -ne 0) { throw 'Could not obtain the executable-contract inventory' }
    $report.artifacts = @($files | Sort-Object -Unique | ForEach-Object {
        $file = Get-Item -LiteralPath $_
        [ordered]@{ path = $_; bytes = $file.Length; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash }
    })
    $report.complete = $true
} catch {
    $failure = $_
    $report.failure = $_.Exception.Message
} finally {
    $reportPath = Join-Path $reportRoot "verification-$run.json"
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
    Pop-Location
    Write-Host "Report: $reportPath"
}
if ($failure) { throw $failure }
