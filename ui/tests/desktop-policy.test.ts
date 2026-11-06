import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = new URL("../../", import.meta.url);
function source(path: string): string { return readFileSync(new URL(path, root), "utf8"); }

// Configuration tests do not claim to test WebView2 or runtime enforcement.
// Rust mock-runtime IPC tests cover actual Tauri command dispatch separately.
describe("minimal desktop exposure", () => {
  const config: unknown = JSON.parse(source("crates/dc-desktop/tauri.conf.json"));
  const capability: unknown = JSON.parse(source("crates/dc-desktop/capabilities/version-page.json"));

  it("selects exactly one local window capability", () => {
    expect(config).toMatchObject({
      identifier: "com.deepcreate.desktop",
      app: {
        withGlobalTauri: false,
        windows: [{ label: "main", create: false, dragDropEnabled: false, devtools: false }],
        security: { capabilities: ["version-page"] }
      }
    });
    expect(config).toHaveProperty("plugins");
    expect(capability).toEqual({
      $schema: "../gen/schemas/desktop-schema.json",
      identifier: "version-page",
      description: "Read build metadata and work units; submit objectives and start attempts. No filesystem, shell, network, database or core plugin permissions.",
      local: true,
      windows: ["main"],
      "permissions": ["allow-application-info", "allow-work-unit-ids", "allow-work-units", "allow-submit-objective", "allow-start-attempt", "allow-plan-objective-graph", "allow-accept-graph", "allow-execute-graph", "allow-health-check"]
    });
    expect(readdirSync(fileURLToPath(new URL("crates/dc-desktop/capabilities/", root)))).toEqual(["version-page.json"]);
  });

  it("requires app ACL instead of Tauri's permissive custom-command default", () => {
    const build = source("crates/dc-desktop/build.rs");
    expect(build).toContain('AppManifest::new().commands(&["application_info", "work_unit_ids", "work_units", "submit_objective", "start_attempt", "plan_objective_graph", "accept_graph", "execute_graph", "health_check"])');
    const native = source("crates/dc-desktop/src/main.rs");
    expect(native).toContain("tauri::generate_handler![");
    for (const cmd of ["application_info", "work_unit_ids", "work_units", "submit_objective", "start_attempt", "plan_objective_graph", "accept_graph", "execute_graph", "health_check"]) {
      expect(native).toContain(cmd);
    }
    expect(native.match(/#\[tauri::command(?:\([^)]*\))?\]/g)).toHaveLength(9);
    expect(native).not.toMatch(/\.plugin\(/);
  });

  it("blocks remote content, browser network, files and broad CSP exceptions", () => {
    expect(config).toMatchObject({
      app: { security: {
        csp: {
          "default-src": "'none'", "connect-src": "ipc: http://ipc.localhost",
          "script-src": "'self'", "style-src": "'self'", "frame-src": "'none'",
          "object-src": "'none'", "worker-src": "'none'", "form-action": "'none'",
          "base-uri": "'none'"
        },
        dangerousDisableAssetCspModification: false,
        assetProtocol: { enable: false, scope: [] }
      } }
    });
    expect(config).toHaveProperty("app.security.csp", {
      "default-src": "'none'",
      "script-src": "'self'",
      "style-src": "'self'",
      "img-src": "'self'",
      "font-src": "'self'",
      "connect-src": "ipc: http://ipc.localhost",
      "object-src": "'none'",
      "frame-src": "'none'",
      "worker-src": "'none'",
      "base-uri": "'none'",
      "form-action": "'none'"
    });
    const text = source("crates/dc-desktop/tauri.conf.json");
    expect(text).not.toMatch(/unsafe-inline|unsafe-eval|\*|devUrl|devCsp|https?:\s/);
    expect(source("crates/dc-desktop/src/main.rs")).toContain(".on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)");
    expect(source("crates/dc-desktop/src/main.rs")).toContain(".on_download(|_, _| false)");
  });

  it("has no native plugin dependencies or frontend data access libraries", () => {
    const manifest: unknown = JSON.parse(source("package.json"));
    expect(manifest).toHaveProperty("dependencies", {
      "@tauri-apps/api": "2.11.1", react: "19.2.8", "react-dom": "19.2.8"
    });
    expect(source("crates/dc-desktop/Cargo.toml")).not.toMatch(/tauri-plugin-|reqwest|rusqlite|hermes/i);
    expect(source("ui/src/application-info.ts")).not.toMatch(/fetch\(|localStorage|indexedDB|WebSocket|XMLHttpRequest/);
  });
});
