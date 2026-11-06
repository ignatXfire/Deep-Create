import { spawnSync } from "node:child_process";
import { readFileSync, readdirSync, existsSync, lstatSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// Document 19 dependency contract for the registered packages only. A new package,
// dependency kind, source, feature or version requirement needs explicit review.
// Cargo.lock, not this direct-edge guard, fixes the complete dependency graph.
export const dependencyContracts = Object.freeze({
  "dc-domain": {
    directory: "crates/dc-domain",
    normal: {
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-01 closed data serialization; no I/O adapter." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-01 closed error types; no runtime service." }
    },
    build: {},
    dev: {
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "Artificial serialization contract tests only." }
    }
  },
  "dc-protocol": {
    directory: "crates/dc-protocol",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-02 reuses closed C-01 command types; no authority or execution." },
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-02 strict typed wire forms." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-02 bounded byte codec, never generic model command execution." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-02 closed sanitized error codes." }
    },
    build: {},
    dev: {}
  },
  "dc-desktop": {
    directory: "crates/dc-desktop",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-14 reuses closed C-01 types for UI projection identifiers." },
      "dc-core": { path: "crates/dc-core", req: "*", features: [], defaults: true, reason: "C-14 UI write path sends owner commands through C-05 handle_command, never touches store directly." },
      "dc-store": { path: "crates/dc-store", req: "*", features: ["sqlite"], defaults: true, reason: "C-14 reads authoritative state from C-03 store for live work-unit projection." },
      "dc-worker": { path: "crates/dc-worker", req: "*", features: [], defaults: true, reason: "C-14 executor dispatches work units through C-11 worker trait." },
      "dc-broker": { path: "crates/dc-broker", req: "*", features: [], defaults: true, reason: "C-14 executor uses C-09 broker endpoint configuration." },
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-14 serialization for Tauri command payloads." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-14 deserializes owner-provided ObjectiveContract JSON at the command boundary." },
      chrono: { req: "^0.4", features: ["serde"], defaults: true, reason: "C-14 timestamp formatting for health_check and UI display." },
      tokio: { req: "^1", features: ["full"], defaults: true, reason: "C-14 async runtime for broker dispatch in executor." },
      tauri: { req: "=2.11.5", features: ["wry", "compression", "common-controls-v6"], defaults: false, reason: "C-14 desktop shell framework." }
    },
    build: {
      "tauri-build": { req: "=2.6.3", features: [], defaults: true, reason: "C-14 build-time Tauri code generation." }
    },
    dev: {
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-14 test JSON payload validation." },
      tauri: { req: "=2.11.5", features: ["test"], defaults: false, reason: "C-14 MockRuntime dispatch tests using the real generated capability context." }
    }
  },
  "dc-store": {
    directory: "crates/dc-store",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-03 reuses closed C-01 entity types; store persists domain objects." },
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-03 entity serialization for persistence." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-03 JSON serialization for stored entities." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-03 closed error types for storage failures." },
      rusqlite: { req: "^0.32", features: ["bundled"], defaults: true, optional: true, reason: "C-03 SQLite backend via bundled libsqlite3-sys; optional feature-gated." }
    },
    build: {},
    dev: {}
  },
  "dc-windows": {
    directory: "crates/dc-windows",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-12 reuses closed C-01 types for Windows admission/fence contracts." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-12 closed error types for Windows adapter failures." },
      "windows-sys": { req: "^0.59", features: ["Win32_System_JobObjects", "Win32_System_Threading", "Win32_Foundation", "Win32_Security", "Win32_Security_Isolation", "Win32_Security_Cryptography", "Win32_System_Memory"], defaults: true, reason: "C-12 real Job Object + AppContainer syscalls (CreateJobObjectW, CreateAppContainerProfile) + DPAPI seal/unseal (CryptProtectData) — single audited unsafe boundary." }
    },
    build: {},
    dev: {}
  },
  "dc-watchdog": {
    directory: "crates/dc-watchdog",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-13 reuses closed C-01 types for watchdog state and fence logic." },
      "dc-store": { path: "crates/dc-store", req: "*", features: ["sqlite"], defaults: true, reason: "C-13 persists authority.anchor via C-03 Stage-3 tables (anti-rollback, WAL+FULL)." },
      "dc-windows": { path: "crates/dc-windows", req: "*", features: [], defaults: true, reason: "C-13 uses C-12 native guards for process supervision." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-13 closed error types for watchdog failures." }
    },
    build: {},
    dev: {}
  },
  "dc-broker": {
    directory: "crates/dc-broker",
    normal: {
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-09 model dispatch request/result serialization." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-09 JSON wire format for dispatch contracts." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-09 closed error types for dispatch failures." },
      reqwest: { req: "^0.12", features: ["json", "rustls-tls"], defaults: false, reason: "C-09 real HTTP dispatch to model endpoints." },
      tokio: { req: "^1", features: ["full"], defaults: true, reason: "C-09 async runtime for HTTP dispatch." }
    },
    build: {},
    dev: {}
  },
  "dc-worker": {
    directory: "crates/dc-worker",
    normal: {
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-11 work-unit execution request/result serialization." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-11 JSON serialization for bounded execution results." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-11 closed error types for execution failures." }
    },
    build: {},
    dev: {
      "dc-broker": { path: "crates/dc-broker", req: "*", features: [], defaults: true, reason: "C-11 live e2e test dispatches through C-09 broker to a real endpoint." }
    }
  },
  "dc-core": {
    directory: "crates/dc-core",
    normal: {
      "dc-domain": { path: "crates/dc-domain", req: "*", features: [], defaults: true, reason: "C-05/C-06/C-07/C-08 reuses closed C-01 domain types; no authority or execution." },
      "dc-protocol": { path: "crates/dc-protocol", req: "*", features: [], defaults: true, reason: "C-05 uses C-02 protocol types for command validation." },
      "dc-store": { path: "crates/dc-store", req: "*", features: ["sqlite"], defaults: true, reason: "C-05 uses C-03 store for atomic state commits and Stage-3 audit records." },
      "dc-worker": { path: "crates/dc-worker", req: "*", features: [], defaults: true, reason: "C-05 uses C-11 worker for bounded execution." },
      serde: { req: "^1.0", features: ["derive"], defaults: true, reason: "C-05 command serialization." },
      serde_json: { req: "^1.0", features: [], defaults: true, reason: "C-05 JSON for command payloads." },
      thiserror: { req: "^2.0", features: [], defaults: true, reason: "C-05 closed error types." }
    },
    build: {},
    dev: {
      "dc-watchdog": { path: "crates/dc-watchdog", req: "*", features: [], defaults: true, reason: "C-05 multi-store consistency integration test with C-13." },
      "dc-broker": { path: "crates/dc-broker", req: "*", features: [], defaults: true, reason: "C-05 live self-dev e2e dispatches through C-09 broker to a real endpoint." }
    }
  }
});

const cratesIoSource = "registry+https://github.com/rust-lang/crates.io-index";
const sorted = (values) => [...values].sort().join("\0");
const issue = (code, packageName, message, details = {}) => ({ code, package: packageName, message, ...details });

/** Read-only validation of cargo metadata's first-party direct dependency edges. */
export function checkMetadata(metadata) {
  if (!metadata || !Array.isArray(metadata.packages) || !Array.isArray(metadata.workspace_members)
      || typeof metadata.workspace_root !== "string") {
    return [issue("INVALID_METADATA", "workspace", "Expected cargo metadata format-version 1.")];
  }
  const errors = [];
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((item) => members.has(item.id));
  if (packages.length !== members.size) errors.push(issue("INCOMPLETE_METADATA", "workspace", "A workspace member has no package description."));
  for (const pkg of packages) {
    const contract = Object.hasOwn(dependencyContracts, pkg.name) ? dependencyContracts[pkg.name] : undefined;
    if (!contract) {
      errors.push(issue("UNREGISTERED_PACKAGE", pkg.name, "Package has no dependency contract from document 19."));
      continue;
    }
    const expected = path.resolve(metadata.workspace_root, contract.directory, "Cargo.toml");
    if (pkg.source !== null || path.resolve(pkg.manifest_path) !== expected) {
      errors.push(issue("PACKAGE_LOCATION", pkg.name, "First-party package is not at its registered workspace path."));
    }
    if (!Array.isArray(pkg.dependencies)) {
      errors.push(issue("INVALID_METADATA", pkg.name, "Missing dependency list."));
      continue;
    }
    for (const dep of pkg.dependencies) {
      const kind = dep.kind ?? "normal";
      const permitted = ["normal", "build", "dev"].includes(kind) && Object.hasOwn(contract[kind], dep.name)
        ? contract[kind][dep.name] : undefined;
      if (!permitted) {
        errors.push(issue("FORBIDDEN_DEPENDENCY", pkg.name, `Unregistered ${kind} dependency: ${dep.name}.`));
        continue;
      }
      const sourceMatches = permitted.path
        ? dep.source === null && typeof dep.path === "string"
          && path.resolve(dep.path) === path.resolve(metadata.workspace_root, permitted.path)
        : dep.source === cratesIoSource && dep.path == null;
      if (!sourceMatches || dep.rename !== null) {
        errors.push(issue("DEPENDENCY_SOURCE", pkg.name, `${dep.name}: path/git/registry substitution or renaming is not registered.`));
      }
      if (dep.optional && !permitted.optional) {
        errors.push(issue("DEPENDENCY_SOURCE", pkg.name, `${dep.name}: optional edge is not registered in the contract.`));
      }
      if (dep.req !== permitted.req || dep.uses_default_features !== permitted.defaults
          || !Array.isArray(dep.features) || sorted(dep.features) !== sorted(permitted.features)) {
        errors.push(issue("DEPENDENCY_CONTRACT", pkg.name, `${dep.name}: version requirement or features changed; review its contract.`));
      }
    }
  }
  return errors;
}

// Small lexical scanner, NOT a Rust parser or macro expander. It skips nested
// comments and string/char/raw-string contents, retaining token line numbers.
function rustTokens(source) {
  const tokens = [];
  let index = 0;
  let line = 1;
  const consume = (end) => {
    line += (source.slice(index, end).match(/\n/g) ?? []).length;
    index = end;
  };
  while (index < source.length) {
    const tail = source.slice(index);
    if (/^\s/.test(tail)) { consume(index + 1); continue; }
    if (tail.startsWith("//")) {
      const end = source.indexOf("\n", index);
      consume(end < 0 ? source.length : end);
      continue;
    }
    if (tail.startsWith("/*")) {
      let end = index + 2;
      let depth = 1;
      while (end < source.length && depth > 0) {
        if (source.startsWith("/*", end)) { depth += 1; end += 2; }
        else if (source.startsWith("*/", end)) { depth -= 1; end += 2; }
        else end += 1;
      }
      consume(end);
      continue;
    }
    const raw = /^(?:b|c)?r(#{0,255})"/.exec(tail);
    if (raw) {
      const close = `"${raw[1]}`;
      const end = source.indexOf(close, index + raw[0].length);
      consume(end < 0 ? source.length : end + close.length);
      continue;
    }
    const quoted = /^(?:b|c)?"(?:[^"\\]|\\[\s\S])*"/.exec(tail)
      ?? /^(?:b)?'(?:[^'\\\n]|\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.))'/.exec(tail);
    if (quoted) { consume(index + quoted[0].length); continue; }
    const token = /^(?:r#)?[A-Za-z_][A-Za-z_0-9]*|^::|^./.exec(tail)?.[0];
    if (!token) { consume(index + 1); continue; }
    tokens.push({ value: token, line });
    consume(index + token.length);
  }
  return tokens;
}

// Expands direct use-trees, including nested std::{fs, net::{...}} forms.
function importedPaths(tokens) {
  const paths = [];
  let cursor = 0;
  const value = () => tokens[cursor]?.value;
  function tree(prefix) {
    const current = [...prefix];
    if (value() === "::") cursor += 1;
    while (cursor < tokens.length) {
      if (value() === "{") {
        cursor += 1;
        while (cursor < tokens.length && value() !== "}") {
          tree(current);
          if (value() === ",") cursor += 1;
          else if (value() !== "}") break;
        }
        if (value() === "}") cursor += 1;
        return;
      }
      if (value() === "as") { cursor += 2; break; }
      if ([",", "}", ";"].includes(value())) break;
      current.push(value());
      cursor += 1;
      if (value() === "::") { cursor += 1; continue; }
      if (value() === "as") cursor += 2;
      break;
    }
    if (current.length > 0) paths.push(current);
  }
  tree([]);
  return paths;
}

const forbiddenStd = new Set(["fs", "net", "time", "env", "process", "thread", "os", "io"]);
const forbiddenMacros = new Set(["env", "option_env", "include", "include_str", "include_bytes", "print", "println", "eprint", "eprintln", "dbg"]);

/** Detect explicit source-level impurity and unsafe; no OS-isolation claim. */
export function checkRustSource({ packageName, file, source, production = true }) {
  const errors = [];
  const tokens = rustTokens(source);
  const seen = new Set();
  const report = (code, token, message) => {
    const key = `${code}:${token.line}:${message}`;
    if (!seen.has(key)) {
      errors.push(issue(code, packageName, message, { file, line: token.line }));
      seen.add(key);
    }
  };
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const name = token.value.replace(/^r#/, "");
    if (token.value === "unsafe" && packageName !== "dc-windows") {
      report("FIRST_PARTY_UNSAFE", token, "unsafe is forbidden outside the registered Windows adapter.");
    }
    if (!["dc-domain", "dc-protocol"].includes(packageName) || !production) continue;
    if (name === "std" && tokens[index + 1]?.value === "::"
        && forbiddenStd.has(tokens[index + 2]?.value.replace(/^r#/, ""))) {
      report("DOMAIN_IMPURE_API", token, `C-01/C-02 data-only code cannot use std::${tokens[index + 2].value}.`);
    }
    if (forbiddenMacros.has(name) && tokens[index + 1]?.value === "!") {
      report("DOMAIN_IMPURE_API", token, `C-01/C-02 data-only code cannot use ${token.value}! external-input/output macros.`);
    }
    if (token.value === "extern") report("DOMAIN_EXTERNAL_CODE", token, "C-01/C-02 data-only code cannot declare extern crates or foreign interfaces.");
    if (token.value === "path" && tokens[index - 1]?.value === "[" && tokens[index + 1]?.value === "=") {
      report("DOMAIN_EXTERNAL_CODE", token, "C-01/C-02 data-only code cannot redirect module sources through #[path].");
    }
    if (token.value !== "use") continue;
    let end = index + 1;
    while (end < tokens.length && tokens[end].value !== ";") end += 1;
    const useTokens = tokens.slice(index + 1, end).map((item) => ({ ...item, value: item.value.replace(/^r#/, "") }));
    for (const imported of importedPaths(useTokens)) {
      if (imported[0] === "std"
          && (imported.length === 1 || ["self", "*"].includes(imported[1])
            || forbiddenStd.has(imported[1]) || forbiddenMacros.has(imported[1]))) {
        report("DOMAIN_IMPURE_IMPORT", token, `C-01/C-02 data-only code cannot import ${imported.join("::")}.`);
      }
    }
  }
  return errors;
}

function scanRustFiles(directory, callback, errors, packageName) {
  if (!existsSync(directory)) return;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      errors.push(issue("SOURCE_SYMLINK", packageName, "Source symlinks need an explicit reviewed scope.", { file }));
    } else if (entry.isDirectory()) {
      // Tests may contain inert Rust snippets. Never skip a src/fixtures module.
      if (entry.name === "fixtures" && path.basename(directory) === "tests") continue;
      scanRustFiles(file, callback, errors, packageName);
    } else if (entry.isFile() && file.endsWith(".rs")) callback(file);
  }
}

export function checkWorkspaceSources(metadata) {
  const errors = [];
  const members = new Set(metadata.workspace_members);
  for (const pkg of metadata.packages.filter((item) => members.has(item.id))) {
    const directory = path.dirname(pkg.manifest_path);
    const collect = (production) => (file) => errors.push(...checkRustSource({
      packageName: pkg.name, file, source: readFileSync(file, "utf8"), production
    }));
    scanRustFiles(path.join(directory, "src"), collect(true), errors, pkg.name);
    scanRustFiles(path.join(directory, "tests"), collect(false), errors, pkg.name);
    scanRustFiles(path.join(directory, "examples"), collect(false), errors, pkg.name);
    const build = path.join(directory, "build.rs");
    if (existsSync(build)) {
      if (lstatSync(build).isSymbolicLink()) errors.push(issue("SOURCE_SYMLINK", pkg.name, "Build script symlink is not registered.", { file: build }));
      else collect(true)(build);
    }
  }
  return errors;
}

function main(args) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  let metadata;
  if (args.length === 2 && args[0] === "--metadata") {
    metadata = JSON.parse(readFileSync(path.resolve(args[1]), "utf8"));
  } else if (args.length === 0) {
    const result = spawnSync("cargo", ["metadata", "--format-version", "1", "--no-deps", "--locked", "--offline"], {
      cwd: root, encoding: "utf8", maxBuffer: 20 * 1024 * 1024, windowsHide: true
    });
    if (result.error || result.status !== 0) throw new Error(result.error?.message ?? result.stderr.trim() ?? "cargo metadata failed");
    metadata = JSON.parse(result.stdout);
  } else throw new Error("Usage: node tools/check-architecture.mjs [--metadata <saved-cargo-metadata.json>]");
  const errors = checkMetadata(metadata);
  if (errors.length === 0 && path.resolve(metadata.workspace_root) !== root) {
    errors.push(issue("WORKSPACE_LOCATION", "workspace", "Saved metadata must belong to this project before any source files are read."));
  }
  if (errors.length === 0) errors.push(...checkWorkspaceSources(metadata));
  process.stdout.write(`${JSON.stringify({
    check: "Document 19 direct dependency and C-01 explicit source guard", ok: errors.length === 0,
    boundary: "Not a full Rust parser, macro expansion, transitive supply-chain audit or OS isolation proof.", errors
  }, null, 2)}\n`);
  process.exitCode = errors.length === 0 ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(process.argv.slice(2)); }
  catch (error) {
    process.stderr.write(`Architecture check failed closed: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
