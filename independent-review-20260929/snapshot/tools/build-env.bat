@echo off
setlocal
set PR=D:/Projects/DeepCreate\.local\hermes-organization-20260916\self-development
set MSVC=%PR%\.local\toolchain\msvc\VC\Tools\MSVC\14.44.35207
set SDK=%PR%\.local\toolchain\sdk\Windows Kits\10
set SDKV=10.0.26100.0
set CBIN=%MSVC%\bin\Hostx64\x64
set SBIN=%SDK%\bin\%SDKV%\x64
set PATH=%CBIN%;%SBIN%;%PATH%
set LIB=%MSVC%\lib\x64;%SDK%\Lib\%SDKV%\um\x64;%SDK%\Lib\%SDKV%\ucrt\x64
set INCLUDE=%MSVC%\include;%SDK%\Include\%SDKV%\ucrt;%SDK%\Include\%SDKV%\shared;%SDK%\Include\%SDKV%\um;%SDK%\Include\%SDKV%\winrt
set CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=%CBIN%\link.exe
set TEMP=%PR%\.local\tmp
set TMP=%PR%\.local\tmp
if not exist "%TEMP%" mkdir "%TEMP%"
cd /d "%PR%"
%*
