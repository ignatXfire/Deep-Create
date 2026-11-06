# Checking the real Windows window

Date: 2026-09-09. The assembled `target/debug/dc-desktop.exe` was checked, not the Vite page.

## What's done

1. The program is launched by a separate process. For this test, the WebView2 store is directed to `.local/native-smoke/webview`, and the connection of the test tool is only allowed by the temporary environment variable of this run. Permanent application and system settings have not changed.
2. Playwright is connected to the real WebView2 program. The page address is `http://tauri.localhost/`, the title is “Deep Create - assembly information.”
3. The page received version **0.1.0**. An additional direct test call to `application_info` returned `{version: "0.1.0", buildStage: "empty_shell"}`.
4. An attempt to call `plugin:fs|read_file` was rejected: `fs.read_file not allowed. Plugin not found`. The user file was not read; There is no file add-on in the assembly.
5. Test `fetch` to the artificial address `https://example.invalid/deep-create-probe` triggered the constraint violation event `connect-src`. It was the content policy ban that was recorded, not just a network failure. Two console errors relate to this intentional negative check.
6. A snapshot of the window is saved and viewed: real window (historical screenshot omitted from this public snapshot). The version and designation of missing functions are displayed without overlay.
7. After the test, it is separately verified that the test process and the debug port 19229 listener no longer exist. The port is not left running.

## What this test does not prove

- This is not a Windows test in a clean virtual machine. Hermes has not been disabled or uninstalled, but the application does not connect to it.
- When the debug port is turned on for testing, you cannot check the resistance of the debugger itself. Normal application startup does not include this port; persistent configuration disables developer tools.
- The absence of a file addition and a single failure test does not prove protection against all vulnerabilities of WebView2 or third-party libraries.
- The base, action mediator, watchman, real model connections and the full work window are not yet implemented in this version.

Information about automatic shell checks: [DESKTOP_STAGE1.md](DESKTOP_STAGE1.md). The overall results of repeated checks are stored `tools/Verify.ps1` separately from this manual check of the present process.
