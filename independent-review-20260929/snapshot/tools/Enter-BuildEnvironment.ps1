# Loads project-local build tools into the current process only.
# Usage: . .\tools\Enter-BuildEnvironment.ps1
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$msvc = Join-Path $projectRoot '.local\toolchain\msvc\VC\Tools\MSVC\14.44.35207'
$sdk = Join-Path $projectRoot '.local\toolchain\sdk\Windows Kits\10'
$sdkVersion = '10.0.26100.0'
$compilerBin = Join-Path $msvc 'bin\Hostx64\x64'
$sdkBin = Join-Path $sdk "bin\$sdkVersion\x64"
$libraries = @(
    (Join-Path $msvc 'lib\x64'),
    (Join-Path $sdk "Lib\$sdkVersion\um\x64"),
    (Join-Path $sdk "Lib\$sdkVersion\ucrt\x64")
)
foreach ($required in @(
    (Join-Path $compilerBin 'link.exe'),
    (Join-Path $libraries[0] 'libcmt.lib'),
    (Join-Path $libraries[1] 'kernel32.lib'),
    (Join-Path $libraries[2] 'ucrt.lib')
)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Build component not found: $required. See docs/implementation/BUILD.md."
    }
}

$env:PATH = "$compilerBin;$sdkBin;$env:PATH"
$env:LIB = $libraries -join ';'
$env:INCLUDE = @(
    (Join-Path $msvc 'include'),
    (Join-Path $sdk "Include\$sdkVersion\ucrt"),
    (Join-Path $sdk "Include\$sdkVersion\shared"),
    (Join-Path $sdk "Include\$sdkVersion\um"),
    (Join-Path $sdk "Include\$sdkVersion\winrt")
) -join ';'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = Join-Path $compilerBin 'link.exe'
$env:VCINSTALLDIR = Join-Path $projectRoot '.local\toolchain\msvc\VC\'
$env:VCToolsInstallDir = "$msvc\"
$env:WindowsSdkDir = "$sdk\"
$env:WindowsSDKVersion = "$sdkVersion\"
$env:VSCMD_ARG_TGT_ARCH = 'x64'
$env:VSLANG = '1049'
$env:TEMP = Join-Path $projectRoot '.local\tmp'
$env:TMP = $env:TEMP
$env:npm_config_cache = Join-Path $projectRoot '.local\npm-cache'
$null = New-Item -ItemType Directory -Path $env:TEMP -Force
