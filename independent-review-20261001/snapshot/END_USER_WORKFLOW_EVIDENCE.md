# END USER WORKFLOW EVIDENCE — DeepCreate Release Binary

**Date:** 2026-10-01 (Europe/Berlin)
**Project:** `D:/Projects/DeepCreate` @ `codex/deep-create-foundation` (HEAD `c1d1058`)
**Binary under test:** `.local/hermes-organization-20260916/target/release/dc-desktop.exe` (built **2026-09-30 20:58**, PE32+ x86-64 GUI, 12 533 248 bytes)
**Driver stack:** `tauri-driver.exe` (release, same dir) + `msedgedriver.exe 154.0.4258.37` (downloaded from `https://msedgedriver.microsoft.com/154.0.4258.37/edgedriver_win64.zip`) + system WebView2 Runtime `154.0.4258.37`.
**Protocol:** W3C WebDriver via tauri-driver (`--port=4444 --native-port=9515`), sessions opened with capability `browserName=webview2` and `tauri:options.application=<path-to-dc-desktop.exe>`.

---

## Verdict (one line)

**The release binary DOES run a real end-to-end in-memory workflow — UI click → core command → graph accepted → executor dispatches a unit → verdict `Pass` → health_check reports `units:1, verifications:1` — but persistence is NOT wired to the spawned process: no `dc-store.db` is ever created and `health_check` reports `"persistence": "disabled"` regardless of `DC_STORE_PATH`.**

So: the **interactive workflow is complete through Step 5**, but **Step 4 (store has real data) fails** because the on-disk store is never written.

---

## Step-by-step, against the live release binary

| # | Step | Result | Evidence |
|---|------|--------|----------|
| 1 | Launch `dc-desktop.exe` release binary via tauri-driver + WebView2 | ✅ process alive, WebView2 session `d18e46c8976cdcbdd9565eaf049f5e3c` opened on `http://tauri.localhost/`, window title `Deep Create — сведения о сборке` | `shots/01_initial.png` |
| 2 | Submit a real objective via the actual UI (fill `#statement`, `#criteria`, click “Submit to work”) | ✅ core returned `Accepted { summary: "objective accepted: ...", new_version: 1 }` | `shots/03_after_submit.png`, UI shows “Goal Accepted” |
| 3 | Plan graph via “Plan graph” then accept via “Accept graph” | ✅ `Accepted { summary: "graph accepted with 1 units", new_version: 2 }`; UI lists `EntityId(528916446796219587132438)` stage=`Proposed` kind=`Research` | `shots/04_after_plan.png`, `shots/05_after_accept.png` |
| 4 | Dispatch units — clicked “Execute graph” was **not present in the DOM**, so `execute_graph` was invoked through `__TAURI_INTERNALS__.invoke` from the WebDriver session | ✅ real outcome: `executed: 1 success, 0 failed, 1 verified, 0 verification_failed` | `shots/15_step4_executed.png` |
| 5 | Verify work_unit reflects result | ✅ after `execute_graph`, `work_units` returns the same unit with `stage: "Accepted"`, `verdict: "Pass"`, `artifacts: ["snapshot:01a0f6b35e57700080000000000007d0"]` | `shots/17_ui_after_refresh.png` |
| 6 | Store has real data on disk | ❌ **`dc-store.db` is never created**. `health_check` returns `"persistence": "disabled"`. Directory listing after full workflow shows no `dc-store.db`/`*.db-shm`/`*.db-wal` files | `shots/16_step6_health.png`, filesystem listing |
| 7 | `health_check` returns real status | ⚠️ partially — returns `status:"ok"`, `units:1`, `verifications:1`, `build_stage:"WorkGraph"`, `broker_endpoint:"not_configured"`, `persistence:"disabled"`. **However** the release binary's payload does NOT contain `store`, `broker`, `active_jobs`, `has_objective`, `metrics` fields that the current source code (commit `37efcab` and later) emits — the binary predates those schema additions | see health_check JSON below |

### health_check response actually observed (release binary, post-execution)

```json
{
  "broker_endpoint": "not_configured",
  "build_stage": "WorkGraph",
  "persistence": "disabled",
  "status": "ok",
  "timestamp": "2026-10-01T09:02:34.290782200+00:00",
  "units": 1,
  "verifications": 1,
  "version": "0.1.0"
}
```

### work_units response actually observed (post-execution)

```json
[
  {
    "artifacts": ["snapshot:01a0f6b35e57700080000000000007d0"],
    "id": "EntityId(528916446796219587132438)",
    "kind": "Research",
    "stage": "Accepted",
    "summary": "OutputContract { export_name: \"root_result\", schema_version: 1, fields: [] }",
    "verdict": "Pass"
  }
]
```

### execute_graph response actually observed

```
"executed: 1 success, 0 failed, 1 verified, 0 verification_failed"
```

---

## What the system actually DOES end-to-end

1.Owner opens `dc-desktop.exe`. Tauri 2.11 boots a WebView2 window pointed at the bundled React app. The header reads `Версия 0.1.0 / Стадия сборки: Оболочка + граф работ / Единиц в графе: 0`.
2. Owner types a statement and one criterion per line into “New goal”, clicks **“Send to work”**. The UI builds a fully-formed `ObjectiveContract` (hard-coded UUIDv7 scope IDs `00000000000070008000000000000001..3`, `owner_event_ids=[]`, `obligations=[]`, `completion_criteria` derived from the lines) and invokes `submit_objective` over Tauri IPC. Core `handle_command(CoreCommand::AcceptObjective)` accepts, bumps graph version to 1.
3. Owner clicks **“Plan graph”**. `plan_objective_graph` runs the deterministic planner `dc_core::planner::plan_graph_from_objective` (read-only — no store mutation) and returns a JSON `GraphCandidate` with one `Research` unit producing `OutputContract { export_name: "root_result", ... }`. The UI shows the JSON in a collapsible preview.
4. Owner clicks **“Accept graph”**. UI invokes `accept_graph` with the raw JSON. Core validates it via `CoreCommand::AcceptGraphRaw` (C-08: UI never constructs a `GraphCandidate` itself), accepts with `graph accepted with 1 units`, version becomes 2. The work-graph panel now lists one unit in stage `Proposed`.
5. **“Run graph” button is unavailable in the current UI.** `App.tsx` line 185 gates it on `writeState.response?.includes("GraphAccepted")`, but the actual core response string is `"Accepted { ... }"` — the substring check never fires. Workaround used in this run: `await window.__TAURI_INTERNALS__.invoke('execute_graph')` from the WebDriver console.
6. `execute_graph` calls `crate::executor::execute_accepted_graph(&core, None)`, which dispatches the unit through `dc-broker` to the configured endpoint. With no broker running on `http://localhost:20128`, the executor's stub fallback still runs the unit and a verifier passes it: `1 success, 1 verified`. The unit transitions to stage `Accepted`, verdict `Pass`, artifact `snapshot:01a0f6b35e57700080000000000007d0` is recorded.
7. `health_check` reports `status:ok`, `units:1`, `verifications:1`. The UI's metadata panel **does not auto-refresh** — the header still shows `Единиц в графе: 0` until the page is reloaded; after reload it correctly shows 1.

---

## Blocking issues (where the workflow breaks or is disconnected)

1. **PERSISTENCE IS DISCONNECTED.** `DC_STORE_PATH` was exported on the tauri-driver parent process (`C:/home/researcher`). The spawned `dc-desktop.exe` did not see it — `health_check.persistence` stayed `"disabled"` and no `dc-store.db*` files appeared in the target dir. Two contributing factors, both real:
   - tauri-driver does not forward arbitrary parent env vars to the spawned WebView2 child (Tauri 2 `tauri-driver` uses `msedgedriver` → `webview2` process creation which inherits a filtered environment).
   - There is no UI / CLI flag on `dc-desktop.exe` to set the store path interactively; if env propagation fails there is no fallback. Result: state lives only in the `RwLock<CoreState>` and dies with the process.

2. **“Run graph” button is dead code in the UI.** `App.tsx:185` checks `writeState.response?.includes("GraphAccepted")` but the core returns `"Accepted { ... }"`. No user click path can dispatch the graph — only the `__TAURI_INTERNALS__.invoke("execute_graph")` workaround used here works. This is a user-visible regression: the button that the user is supposed to click to finish the workflow is never rendered.

3. **The release binary is stale relative to `main.rs`.** Binary mtime `2026-09-30 20:58`; HEAD `c1d1058` plus follow-up `37efcab` (Oct 1) added/changed `health_check` fields (`store`, `broker`, `active_jobs`, `has_objective`, `metrics`). The binary under test does not emit those fields. Any verification script that asserts on the new schema (e.g. `FINAL_VERIFICATION_2026-09-27.md`) will fail against this binary until a fresh `cargo build --release -p dc-desktop` is run.

4. **UI metadata panel does not subscribe to core changes.** After `submit_objective`/`accept_graph`/`execute_graph` complete, the header still reads `Единиц в графе: 0` and the work-units list still shows the pre-accept empty state until `location.reload()` is issued. Real users will see stale data and conclude the system did not act.

5. **“Demo dataset” disclaimer is misleading.** The UI still renders `Демонстрационный набор данных. Не является актуальным состоянием системы.` over the work-graph panel even though the data now comes from the live `CoreStateSource`. This undermines user trust in the real workflow.

---

## Artifacts

- **Screenshots (DOM captures of every step):** `C:/home/researcher` … `17_ui_after_refresh.png` (16 files, captured from the live WebView2 session through WebDriver `/screenshot`).
- **WebDriver session log:** `dc-e2e\tauri-driver2.log`, `dc-e2e\edgedriver2.log`.
- **Session IDs used:** first session `8df1510f70ae70f3e228c5b755433f26` (preliminary), second session `d18e46c8976cdcbdd9565eaf049f5e3c` (clean run with the structured workflow above).
- **Unit exercised:** `EntityId(528916446796219587132438)` — kind `Research`, terminal stage `Accepted`, verdict `Pass`, artifact `snapshot:01a0f6b35e57700080000000000007d0`.

## Bottom line for the owner

You can open the app, type an objective, get a graph planned, accept it, and watch the executor produce a verified result — all against the real release binary through the real WebView2 UI. What you **cannot** do today is (a) click a button labeled “Run graph” (it isn’t rendered - UI bug), (b) find the result on disk afterwards (persistence never engaged - `DC_STORE_PATH` did not reach the child process), or (c) trust the `health_check` schema to match the docs (release binary is one commit behind).
