#Requires -Version 5.1
<#!
Prepares the already selected compiler/SDK payloads inside this D: checkout.
No global installation or profile edits. Stop builds before preparation.
-ValidateOnly checks sources/planned paths without extraction, MSI, or reports.
-AllowDownload permits only the pinned CRT headers URL when its local file is absent.
#>
[CmdletBinding()]
param(
    [string]$CacheRoot = 'C:\ProgramData\Microsoft\VisualStudio\Packages',
    [switch]$AllowDownload,
    [switch]$ValidateOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..')).TrimEnd('\')
if ([IO.Path]::GetPathRoot($projectRoot) -ine 'D:\') { throw 'This helper is restricted to a checkout on D:.' }
if ($ValidateOnly -and $AllowDownload) { throw '-ValidateOnly cannot be combined with -AllowDownload.' }
$toolchainRoot = Join-Path $projectRoot '.local\toolchain'
$msvcRoot = Join-Path $toolchainRoot 'msvc'
$sdkRoot = Join-Path $toolchainRoot 'sdk'
$headersPath = Join-Path $toolchainRoot 'crt-headers.vsix'
$headersHash = '852382A9AA73502B7849C1BCADFB603BA7175C4E8B60E6ABA03C7DE711D4ECE5'
$headersUrl = 'https://download.visualstudio.microsoft.com/download/pr/c610cd8c-801b-44b8-a80a-82cc382aeb43/852382a9aa73502b7849c1bcadfb603ba7175c4e8b60e6aba03c7de711d4ece5/Microsoft.VC.14.44.17.14.CRT.Headers.base.vsix'
$sdkInstallers = Join-Path $CacheRoot 'Win11SDK_10.0.26100,version=10.0.26100.15,productarch=neutral\Installers'
$vsixInputs = @(
    @{ Name = 'Tools.HostX64.TargetX64.base'; RelativePath = 'Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base,version=14.44.35223,chip=x64\payload.vsix'; Hash = 'CC1C0D0F44F17EC28CF9F9755A40AD6C5B73B35538ED593CB931CFCE5D32D2D3' },
    @{ Name = 'Tools.HostX64.TargetX64.Res.ru-RU'; RelativePath = 'Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.Res.base,version=14.44.35223,chip=x64,language=ru-RU\payload.vsix'; Hash = 'D87D38B407734385FA879C174D844DCFC3E7E89275EBC8341FF0EB0181A3E815' },
    @{ Name = 'CRT.x64.Desktop.base'; RelativePath = 'Microsoft.VC.14.44.17.14.CRT.x64.Desktop.base,version=14.44.35222\payload.vsix'; Hash = '3D6B398F32EAFD7FB3735F83402C0658D831C3B4EE10F93F0031F7A78F311C27' },
    @{ Name = 'CRT.x64.Store.base'; RelativePath = 'Microsoft.VC.14.44.17.14.CRT.x64.Store.base,version=14.44.35222\payload.vsix'; Hash = '2FEA29C858FC487873E813926D240C97235153C2BEF82BF22D17CCE45E7BCF57' }
)
$msiNames = @(
    'Windows SDK Desktop Libs x64-x86_en-us.msi',
    'Universal CRT Headers Libraries and Sources-x86_en-us.msi',
    'Windows SDK Desktop Tools x64-x86_en-us.msi',
    'Windows SDK for Windows Store Apps Libs-x86_en-us.msi',
    'Windows SDK for Windows Store Apps Headers-x86_en-us.msi',
    'Windows SDK for Windows Store Apps Headers OnecoreUap-x86_en-us.msi',
    'Windows SDK Desktop Headers x86-x86_en-us.msi',
    'Windows SDK Desktop Headers x64-x86_en-us.msi',
    'Windows SDK for Windows Store Apps Tools-x86_en-us.msi'
)

function Assert-ProjectPath([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    if ($full -ine $projectRoot -and -not $full.StartsWith($projectRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw "Output escaped the checkout: $full"
    }
    # Check existing ancestors, including the checkout itself; never follow a
    # junction/symlink into an unrelated directory during preparation.
    $cursor = $full
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Reparse point in output path: $cursor" }
        }
        $parent = [IO.Path]::GetDirectoryName($cursor)
        if ($parent -eq $cursor) { break }
        $cursor = $parent
    }
    return $full
}

function Ensure-Directory([string]$Path) {
    $full = Assert-ProjectPath $Path
    $null = [IO.Directory]::CreateDirectory($full)
}

function Get-Sha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Get-StreamSha256([IO.Stream]$Stream) {
    $hash = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($hash.ComputeHash($Stream))).Replace('-', '') }
    finally { $hash.Dispose() }
}

function Get-ExternalCabinets([string]$MsiPath) {
    # Read-only MSI database access; this does not run installer actions.
    $installer = $null; $database = $null; $view = $null; $record = $null
    try {
        $installer = New-Object -ComObject WindowsInstaller.Installer
        $database = $installer.GetType().InvokeMember('OpenDatabase', [Reflection.BindingFlags]::InvokeMethod, $null, $installer, @($MsiPath, 0))
        $view = $database.GetType().InvokeMember('OpenView', [Reflection.BindingFlags]::InvokeMethod, $null, $database, @('SELECT `Cabinet` FROM `Media`'))
        $null = $view.GetType().InvokeMember('Execute', [Reflection.BindingFlags]::InvokeMethod, $null, $view, $null)
        while ($null -ne ($record = $view.GetType().InvokeMember('Fetch', [Reflection.BindingFlags]::InvokeMethod, $null, $view, $null))) {
            $cabinet = $record.GetType().InvokeMember('StringData', [Reflection.BindingFlags]::GetProperty, $null, $record, @(1))
            if ($cabinet -and -not $cabinet.StartsWith('#')) {
                if ($cabinet -notmatch '^[A-Za-z0-9][A-Za-z0-9_.-]*\.cab$' -or $cabinet.Contains('..')) { throw "Unsafe MSI cabinet name: $cabinet" }
                $cabinet
            }
            $null = [Runtime.InteropServices.Marshal]::FinalReleaseComObject($record)
            $record = $null
        }
        $null = $view.GetType().InvokeMember('Close', [Reflection.BindingFlags]::InvokeMethod, $null, $view, $null)
    }
    finally {
        foreach ($item in @($record, $view, $database, $installer)) {
            if ($null -ne $item) { $null = [Runtime.InteropServices.Marshal]::FinalReleaseComObject($item) }
        }
    }
}

function Resolve-ArchiveDestination([string]$EntryName) {
    $name = $EntryName.Replace('\', '/')
    if (-not $name.StartsWith('Contents/', [StringComparison]::Ordinal)) { return $null }
    $relative = $name.Substring('Contents/'.Length).TrimEnd('/')
    if (-not $relative) { return $null }
    foreach ($part in $relative.Split('/')) {
        if (-not $part -or $part -eq '.' -or $part -eq '..' -or $part -match '[:<>"|?*\x00-\x1F]' -or $part.EndsWith('.') -or $part.EndsWith(' ') -or
            $part -match '^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)') { throw "Unsafe ZIP entry: $EntryName" }
    }
    if (-not $relative.StartsWith('VC/Tools/MSVC/14.44.35207/', [StringComparison]::Ordinal)) { throw "Unexpected MSVC payload root: $EntryName" }
    $destination = Assert-ProjectPath (Join-Path $msvcRoot $relative.Replace('/', '\'))
    if (-not $destination.StartsWith($msvcRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw "ZIP traversal: $EntryName" }
    return $destination
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$archives = New-Object 'System.Collections.Generic.List[System.IDisposable]'
$sourceReport = New-Object 'System.Collections.Generic.List[object]'
$plans = @{}
$msiInputs = New-Object 'System.Collections.Generic.List[object]'
$cabinetInputs = @{}
$lock = $null
try {
    $null = Assert-ProjectPath $toolchainRoot
    foreach ($input in $vsixInputs) {
        $path = Join-Path $CacheRoot $input.RelativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required cached VSIX missing: $path" }
        if ((Get-Sha256 $path) -ne $input.Hash) { throw "Pinned VSIX hash mismatch: $path" }
        $input.Path = $path
    }
    # Validate every MSI and its external cabinet before running any /a action.
    foreach ($name in $msiNames) {
        $path = Join-Path $sdkInstallers $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required cached MSI missing: $path" }
        $signature = Get-AuthenticodeSignature -LiteralPath $path
        if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '(?:^|,\s*)O=Microsoft Corporation(?:,|$)') {
            throw "MSI needs a valid Microsoft signature: $path ($($signature.Status))"
        }
        $msiInputs.Add([pscustomobject]@{ Name = $name; Path = $path; Sha256 = (Get-Sha256 $path); SignatureStatus = [string]$signature.Status; Signer = $signature.SignerCertificate.Subject })
        foreach ($cabinet in @(Get-ExternalCabinets $path)) {
            $cabPath = Join-Path $sdkInstallers $cabinet
            if (-not (Test-Path -LiteralPath $cabPath -PathType Leaf)) { throw "Required SDK cabinet missing: $cabPath" }
            if (-not $cabinetInputs.ContainsKey($cabPath)) { $cabinetInputs[$cabPath] = [pscustomobject]@{ Name = $cabinet; Path = $cabPath; Sha256 = (Get-Sha256 $cabPath) } }
        }
    }
    if (-not (Test-Path -LiteralPath $headersPath -PathType Leaf)) {
        if (-not $AllowDownload) { throw 'Pinned CRT headers are absent. Supply .local/toolchain/crt-headers.vsix or explicitly pass -AllowDownload.' }
        Ensure-Directory $toolchainRoot
        $download = Assert-ProjectPath (Join-Path $toolchainRoot ('crt-headers-' + [Guid]::NewGuid().ToString('N') + '.download'))
        try {
            # No alternate address, automatic redirect, or hash-mismatch fallback.
            Invoke-WebRequest -Uri $headersUrl -OutFile $download -MaximumRedirection 0 -UseBasicParsing
            if ((Get-Sha256 $download) -ne $headersHash) { throw 'Downloaded CRT headers do not match the pinned SHA256.' }
            $null = Assert-ProjectPath $headersPath
            [IO.File]::Move($download, $headersPath)
        }
        finally { if (Test-Path -LiteralPath $download -PathType Leaf) { $null = Assert-ProjectPath $download; [IO.File]::Delete($download) } }
    }
    $allVsix = @($vsixInputs) + @(@{ Name = 'CRT.Headers.base'; Path = $headersPath; Hash = $headersHash })
    $expandedBytes = [uint64]0
    foreach ($input in $allVsix) {
        # Keep read-only handles open from hash verification through extraction.
        $stream = [IO.File]::Open($input.Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        $archives.Add($stream)
        if ((Get-StreamSha256 $stream) -ne $input.Hash) { throw "Pinned VSIX hash mismatch: $($input.Path)" }
        $stream.Position = 0
        $zip = New-Object IO.Compression.ZipArchive($stream, [IO.Compression.ZipArchiveMode]::Read, $true)
        $archives.Add($zip)
        $sourceReport.Add([pscustomobject]@{ Name = $input.Name; Path = $input.Path; Sha256 = $input.Hash })
        $contentsCount = 0
        foreach ($entry in $zip.Entries) {
            $destination = Resolve-ArchiveDestination $entry.FullName
            if (-not $destination) { continue }
            if ((($entry.ExternalAttributes -shr 16) -band 0xF000) -eq 0xA000 -or ($entry.ExternalAttributes -band 0x400) -ne 0) { throw "Link-like ZIP entry: $($entry.FullName)" }
            if ($entry.FullName.EndsWith('/') -or $entry.FullName.EndsWith('\')) { continue }
            if ($entry.Length -lt 0 -or $entry.Length -gt 1GB -or $expandedBytes -gt (4GB - [uint64]$entry.Length)) { throw 'VSIX expanded-size limit exceeded.' }
            $expandedBytes += [uint64]$entry.Length
            $contentsCount++
            if ($plans.Count -ge 100000) { throw 'VSIX entry-count limit exceeded.' }
            $entryStream = $entry.Open()
            try { $entryHash = Get-StreamSha256 $entryStream } finally { $entryStream.Dispose() }
            if ($plans.ContainsKey($destination)) {
                if ($plans[$destination].Sha256 -ne $entryHash) { throw "Conflicting payloads for $destination" }
            }
            else { $plans[$destination] = [pscustomobject]@{ Destination = $destination; Entry = $entry; Sha256 = $entryHash } }
        }
        if ($contentsCount -eq 0) { throw "VSIX contains no expected Contents files: $($input.Path)" }
    }
    $null = Assert-ProjectPath $sdkRoot
    if (Test-Path -LiteralPath $sdkRoot -PathType Container) {
        foreach ($item in Get-ChildItem -LiteralPath $sdkRoot -Force -Recurse) {
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Reparse point inside SDK destination: $($item.FullName)" }
        }
    }
    if ($ValidateOnly) {
        [pscustomobject]@{ Mode = 'ValidatedOnly'; VsixCount = $allVsix.Count; MsiCount = $msiInputs.Count; ExternalCabinetCount = $cabinetInputs.Count; PlannedFileCount = $plans.Count; ExpandedBytes = $expandedBytes; ExtractionExecuted = $false; MsiExecuted = $false }
        return
    }

    Ensure-Directory $toolchainRoot
    $lockPath = Assert-ProjectPath (Join-Path $toolchainRoot 'preparation.lock')
    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    foreach ($plan in $plans.Values | Sort-Object Destination) {
        $destination = Assert-ProjectPath $plan.Destination
        if ((Test-Path -LiteralPath $destination -PathType Leaf) -and (Get-Sha256 $destination) -eq $plan.Sha256) { continue }
        Ensure-Directory ([IO.Path]::GetDirectoryName($destination))
        $temporary = Assert-ProjectPath ($destination + '.' + [Guid]::NewGuid().ToString('N') + '.prepare')
        try {
            $source = $plan.Entry.Open()
            $output = [IO.File]::Open($temporary, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
            try { $source.CopyTo($output) } finally { $output.Dispose(); $source.Dispose() }
            if ((Get-Sha256 $temporary) -ne $plan.Sha256) { throw "Extracted payload hash mismatch: $destination" }
            $null = Assert-ProjectPath $destination
            if (Test-Path -LiteralPath $destination -PathType Leaf) { [IO.File]::Replace($temporary, $destination, $null) }
            else { [IO.File]::Move($temporary, $destination) }
        }
        finally { if (Test-Path -LiteralPath $temporary -PathType Leaf) { $null = Assert-ProjectPath $temporary; [IO.File]::Delete($temporary) } }
    }
    Ensure-Directory $sdkRoot
    $logsRoot = Join-Path $toolchainRoot 'preparation-logs'
    Ensure-Directory $logsRoot
    $runId = [Guid]::NewGuid().ToString('N')
    $msiexec = Join-Path $env:SystemRoot 'System32\msiexec.exe'
    foreach ($msi in $msiInputs) {
        if ((Get-Sha256 $msi.Path) -ne $msi.Sha256) { throw "MSI changed after preflight: $($msi.Path)" }
        $null = Assert-ProjectPath $sdkRoot
        $log = Assert-ProjectPath (Join-Path $logsRoot ($runId + '-' + [IO.Path]::GetFileNameWithoutExtension($msi.Name) + '.log'))
        $arguments = '/a "{0}" /qn TARGETDIR="{1}" /norestart /L*v "{2}"' -f $msi.Path, $sdkRoot, $log
        $process = Start-Process -FilePath $msiexec -ArgumentList $arguments -WindowStyle Hidden -PassThru -Wait
        if ($process.ExitCode -ne 0) { throw "Administrative SDK extraction failed (exit $($process.ExitCode)); see $log. No success report was issued." }
    }
    $requiredFiles = @(
        'msvc\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe',
        'msvc\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\cl.exe',
        'msvc\VC\Tools\MSVC\14.44.35207\include\vcruntime.h',
        'msvc\VC\Tools\MSVC\14.44.35207\lib\x64\libcmt.lib',
        'sdk\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib',
        'sdk\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64\ucrt.lib',
        'sdk\Windows Kits\10\Include\10.0.26100.0\um\Windows.h',
        'sdk\Windows Kits\10\Include\10.0.26100.0\ucrt\stdio.h',
        'sdk\Windows Kits\10\bin\10.0.26100.0\x64\rc.exe',
        'sdk\Windows Kits\10\bin\10.0.26100.0\x64\mt.exe'
    )
    $outputs = foreach ($relative in $requiredFiles) {
        $path = Assert-ProjectPath (Join-Path $toolchainRoot $relative)
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required prepared output missing: $path" }
        [pscustomobject]@{ RelativePath = $relative; Sha256 = (Get-Sha256 $path) }
    }
    $reportsRoot = Join-Path $toolchainRoot 'preparation-reports'
    Ensure-Directory $reportsRoot
    $sequence = [uint64]0
    foreach ($file in Get-ChildItem -LiteralPath $reportsRoot -File) {
        if ($file.Name -match '^preparation-([0-9]{20})\.json(?:\.pending)?$') {
            $seen = [uint64]::Parse($Matches[1])
            if ($seen -gt $sequence) { $sequence = $seen }
        }
    }
    if ($sequence -eq [uint64]::MaxValue) { throw 'Preparation report sequence exhausted.' }
    $sequence++
    $reportPath = Assert-ProjectPath (Join-Path $reportsRoot ('preparation-{0:D20}.json' -f $sequence))
    $pending = Assert-ProjectPath ($reportPath + '.pending')
    $report = [ordered]@{ SchemaVersion = 1; Sequence = $sequence.ToString(); Status = 'Prepared'; PreparedAtUtc = [DateTime]::UtcNow.ToString('o'); ProjectRoot = $projectRoot; Vsix = @($sourceReport); SdkMsi = @($msiInputs); SdkCabinets = @($cabinetInputs.Values | Sort-Object Name); RequiredOutputs = @($outputs); AdministrativeExtraction = $true; GlobalInstallationRequested = $false; BuildExecuted = $false }
    $bytes = [Text.Encoding]::UTF8.GetBytes(($report | ConvertTo-Json -Depth 8))
    $reportStream = [IO.File]::Open($pending, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $reportStream.Write($bytes, 0, $bytes.Length); $reportStream.Flush($true) } finally { $reportStream.Dispose() }
    [IO.File]::Move($pending, $reportPath)
    Write-Output "Prepared project-local MSVC/SDK. Immutable report: $reportPath"
}
finally {
    if ($null -ne $lock) { $lock.Dispose() }
    for ($i = $archives.Count - 1; $i -ge 0; $i--) { $archives[$i].Dispose() }
}
