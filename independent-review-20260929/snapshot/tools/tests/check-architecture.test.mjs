import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { checkMetadata, checkRustSource, dependencyContracts } from "../check-architecture.mjs";

function metadataFixture() {
  const root = path.resolve("synthetic-workspace");
  const packages = Object.entries(dependencyContracts).map(([name, contract]) => ({
    id: `path+synthetic#${name}@0.1.0`, name, source: null,
    manifest_path: path.join(root, contract.directory, "Cargo.toml"),
    dependencies: Object.entries(contract).flatMap(([kind, deps]) => {
      if (!["normal", "build", "dev"].includes(kind)) return [];
      return Object.entries(deps).map(([dependency, rule]) => ({
        name: dependency, kind: kind === "normal" ? null : kind,
        source: rule.path ? null : "registry+https://github.com/rust-lang/crates.io-index",
        ...(rule.path ? { path: path.resolve(root, rule.path) } : {}),
        req: rule.req, features: rule.features, uses_default_features: rule.defaults,
        rename: null, optional: false
      }));
    })
  }));
  return { workspace_root: root, packages, workspace_members: packages.map((pkg) => pkg.id) };
}

const codes = (issues) => issues.map((item) => item.code);
function source(code, production = true, packageName = "dc-domain") {
  return checkRustSource({ packageName, file: "synthetic.rs", source: code, production });
}

test("the registered existing package contracts pass", () => {
  assert.deepEqual(checkMetadata(metadataFixture()), []);
});

test("a new workspace package is rejected, not silently ignored", () => {
  const data = metadataFixture();
  data.packages.push({ ...data.packages[0], id: "new", name: "dc-new" });
  data.workspace_members.push("new");
  assert.ok(codes(checkMetadata(data)).includes("UNREGISTERED_PACKAGE"));
});

test("object prototype names cannot masquerade as a registered package or dependency", () => {
  const data = metadataFixture();
  data.packages[0].dependencies.push({ name: "toString", kind: null });
  assert.ok(codes(checkMetadata(data)).includes("FORBIDDEN_DEPENDENCY"));
  data.packages[0].name = "constructor";
  assert.ok(codes(checkMetadata(data)).includes("UNREGISTERED_PACKAGE"));
});

for (const [owner, dependency] of [["dc-domain", "dc-store"], ["dc-domain", "reqwest"], ["dc-desktop", "dc-store"]]) {
  test(`${owner} cannot add ${dependency}`, () => {
    const data = metadataFixture();
    data.packages.find((pkg) => pkg.name === owner).dependencies.push({ name: dependency, kind: null });
    assert.ok(codes(checkMetadata(data)).includes("FORBIDDEN_DEPENDENCY"));
  });
}

test("build and dev dependencies do not bypass the explicit allowlist", () => {
  const data = metadataFixture();
  data.packages[0].dependencies.push({ name: "reqwest", kind: "build" }, { name: "tokio", kind: "dev" });
  assert.equal(checkMetadata(data).filter((entry) => entry.code === "FORBIDDEN_DEPENDENCY").length, 2);
});

test("features/defaults/version requirements cannot silently expand", () => {
  for (const change of [{ features: ["derive", "extra"] }, { uses_default_features: false }, { req: "*" }]) {
    const data = metadataFixture();
    Object.assign(data.packages[0].dependencies[0], change);
    assert.ok(codes(checkMetadata(data)).includes("DEPENDENCY_CONTRACT"));
  }
});

test("path/git substitutions and aliases cannot masquerade as an allowed crate", () => {
  for (const change of [{ source: null, path: "/untrusted" }, { source: "git+https://example.invalid/serde" }, { rename: "io" }, { optional: true }]) {
    const data = metadataFixture();
    Object.assign(data.packages[0].dependencies[0], change);
    assert.ok(codes(checkMetadata(data)).includes("DEPENDENCY_SOURCE"));
  }
});

test("invalid and incomplete metadata fails closed", () => {
  assert.deepEqual(codes(checkMetadata({})), ["INVALID_METADATA"]);
  const data = metadataFixture();
  data.workspace_members.push("missing-member");
  assert.ok(codes(checkMetadata(data)).includes("INCOMPLETE_METADATA"));
});

test("protocol's local domain edge rejects path, registry and alias substitution", () => {
  for (const change of [{ path: path.resolve("other-domain") },
    { source: "registry+https://github.com/rust-lang/crates.io-index" },
    { rename: "other" }, { optional: true }]) {
    const data = metadataFixture();
    const edge = data.packages.find((p) => p.name === "dc-protocol").dependencies.find((d) => d.name === "dc-domain");
    Object.assign(edge, change);
    assert.ok(codes(checkMetadata(data)).includes("DEPENDENCY_SOURCE"));
  }
});

test("protocol cannot import storage or network through any dependency kind", () => {
  for (const kind of [null, "dev", "build"]) {
    const data = metadataFixture();
    data.packages.find((p) => p.name === "dc-protocol").dependencies.push({ name: "dc-store", kind });
    assert.ok(codes(checkMetadata(data)).includes("FORBIDDEN_DEPENDENCY"));
  }
});

test("protocol's byte codec remains data-only", () => {
  for (const code of ["use std::fs;", "use std::net::TcpStream;", "fn x() { std::env::var(\"X\"); }"]) {
    assert.ok(source(code, true, "dc-protocol").length > 0);
  }
});

test("safe data-only standard-library imports pass", () => {
  assert.deepEqual(source("use std::{collections::{BTreeMap, BTreeSet}, num::NonZeroU32, fmt}; fn x() {}"), []);
});

for (const code of [
  "use std::fs::File;", "use std::{collections::BTreeMap, net::{TcpStream}};",
  "use ::std::{fs as files};", "use std as platform;", "use std::*;",
  "use r#std::{r#fs as files};", "use std::{println as log};",
  "fn x() { std /* comment */ :: time :: SystemTime::now(); }",
  "fn x() { std::env::var(\"NAME\"); }", "fn x() { std::process::Command::new(\"cmd\"); }",
  "fn x() { println!(\"data\"); }", "const X: &str = env!(\"HOME\");",
  "include!(\"external.rs\");", "extern crate std as other;", '#[path = "outside.rs"] mod hidden;'
]) {
  test(`C-01 explicit impurity detected: ${code}`, () => { assert.ok(source(code).length > 0); });
}

test("actual first-party unsafe is rejected, including test code", () => {
  assert.ok(codes(source("unsafe { call(); }", false, "dc-desktop")).includes("FIRST_PARTY_UNSAFE"));
  assert.ok(codes(source("unsafe fn x() {}", true)).includes("FIRST_PARTY_UNSAFE"));
});

test("inert strings, raw strings, character literals and comments are not code", () => {
  const snippets = [
    '// unsafe { std::fs::read("x"); }',
    '/* unsafe /* nested */ std::net::TcpStream */',
    'let fixture = "unsafe { std::env::var(\\\"x\\\"); }";',
    'let fixture = r###"unsafe { std::fs::File }"###;',
    'let fixture = br#"unsafe std::net"#;',
    "let quote = '\\''; let letter = 'a';"
  ];
  for (const code of snippets) assert.deepEqual(source(code), [], code);
});

test("test harness I/O is outside C-01 runtime-purity scanning, not unsafe rules", () => {
  assert.deepEqual(source('use std::fs; fn test_fixture() { println!("test"); }', false), []);
});

test("line numbers preserve skipped comments and strings", () => {
  const findings = source('// comment\nlet text = "literal";\nunsafe {}');
  assert.equal(findings[0].line, 3);
});
