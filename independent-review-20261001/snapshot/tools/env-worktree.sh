# Process-local build environment for THIS worktree only.
# Sources the original (read-only) cached MSVC/SDK toolchain and redirects all
# build artifacts onto D inside this worktree. No global/system changes.
# Usage (git-bash):  source tools/env-worktree.sh

_WS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
_TOOLCHAIN="${DEEP_CREATE_TOOLCHAIN_ROOT:-$_WS_ROOT/.local/toolchain}"
_MSVC="$_TOOLCHAIN/msvc/VC/Tools/MSVC/14.44.35207"
_SDK="$_TOOLCHAIN/sdk/Windows Kits/10"
_SDKV="10.0.26100.0"

[ -f "$_MSVC/bin/Hostx64/x64/link.exe" ] || { echo "missing MSVC link.exe"; return 1; }
[ -f "$_SDK/Lib/$_SDKV/um/x64/kernel32.lib" ] || { echo "missing SDK libs"; return 1; }

# Native Windows-style variants for tools (cc-rs, link.exe) that need them.
_MSVC_WIN="$(cygpath -w "$_MSVC")"
_SDK_WIN="$(cygpath -w "$_SDK")"

export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="$_MSVC_WIN\\bin\\Hostx64\\x64\\link.exe"
# PATH must be MSYS-style (/d/...) so the bash PATH separator `:` is not
# confused with the drive-letter colon in `D:`.
export PATH="$_MSVC/bin/Hostx64/x64:$_SDK/bin/$_SDKV/x64:$PATH"
# LIB and INCLUDE are consumed by cl.exe / link.exe (Windows tools): use `;`.
export LIB="$_MSVC_WIN\\lib\\x64;$_SDK_WIN\\Lib\\$_SDKV\\um\\x64;$_SDK_WIN\\Lib\\$_SDKV\\ucrt\\x64"
export INCLUDE="$_MSVC_WIN\\include;$_SDK_WIN\\Include\\$_SDKV\\ucrt;$_SDK_WIN\\Include\\$_SDKV\\shared;$_SDK_WIN\\Include\\$_SDKV\\um;$_SDK_WIN\\Include\\$_SDKV\\winrt"
export CARGO_TARGET_DIR="$_WS_ROOT/target"
# vswhom-sys / cc-based build scripts probe these.
export VCINSTALLDIR="$(cygpath -w "$_TOOLCHAIN/msvc/VC")\\"
export VCToolsInstallDir="$_MSVC_WIN\\"
export WindowsSdkDir="$_SDK_WIN\\"
export WindowsSDKVersion="$_SDKV\\"
export VSCMD_ARG_TGT_ARCH=x64
# rustup/cargo caches stay in the user profile (no global change); target is worktree-local.
unset _WS_ROOT _TOOLCHAIN _MSVC _SDK _SDKV _MSVC_WIN _SDK_WIN
