// Real SQLite conditional-update/transaction tests of the narrow head contract.
// The fixture graph/event tables are NOT the production store or its schema.
import test from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { readFileSync, mkdirSync, mkdtempSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const schema = readFileSync(new URL("../../docs/implementation/contracts/topology-head.sql", import.meta.url), "utf8");
const owner = Buffer.from("019f0000000070008000000000000001", "hex");
const otherOwner = Buffer.from("019f0000000070008000000000000002", "hex");
function bytes(value) { const result = Buffer.alloc(8); result.writeBigUInt64BE(value); return result; }
function fixture(t) {
  const parent = path.resolve(root, ".local", "topology-contract-tests");
  assert.ok(parent.startsWith(path.resolve(root) + path.sep));
  mkdirSync(parent, { recursive: true });
  const directory = mkdtempSync(path.join(parent, "run-"));
  // Retained synthetic files only, no recursive deletion of user directories.
  const location = path.join(directory, "fixture.db");
  const writer = new DatabaseSync(location);
  writer.exec("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;");
  writer.exec(schema);
  writer.exec("CREATE TABLE fixture_edges (a TEXT, b TEXT, PRIMARY KEY(a,b)) STRICT; CREATE TABLE fixture_events (id INTEGER PRIMARY KEY, revision BLOB) STRICT;");
  writer.prepare("INSERT INTO graph_topology_head VALUES (1, ?, ?)").run(owner, bytes(7n));
  const observer = new DatabaseSync(location);
  t.after(() => { observer.close(); writer.close(); });
  return { writer, observer };
}
function readHead(db) {
  return Buffer.from(db.prepare("SELECT revision FROM graph_topology_head WHERE singleton=1").get().revision).readBigUInt64BE();
}
function commit(db, expected, a, b, { failAfterEdge = false, installation = owner } = {}) {
  const next = expected + 1n;
  db.exec("BEGIN IMMEDIATE");
  try {
    const changed = db.prepare("UPDATE graph_topology_head SET revision=? WHERE singleton=1 AND installation_id=? AND revision=?")
      .run(bytes(next), installation, bytes(expected)).changes;
    if (changed !== 1) throw new Error("GraphVersionConflict");
    db.prepare("INSERT INTO fixture_edges VALUES (?,?)").run(a, b);
    if (failAfterEdge) throw new Error("InjectedFailure");
    db.prepare("INSERT INTO fixture_events(revision) VALUES (?)").run(bytes(next));
    db.exec("COMMIT");
  } catch (error) { db.exec("ROLLBACK"); throw error; }
}

test("two stale opposite-edge proposals cannot both commit", (t) => {
  const { writer, observer } = fixture(t);
  const first = readHead(writer), second = readHead(observer);
  commit(writer, first, "A", "B");
  assert.throws(() => commit(writer, second, "B", "A"), /GraphVersionConflict/);
  assert.equal(readHead(observer), 8n);
  assert.equal(observer.prepare("SELECT count(*) AS n FROM fixture_edges").get().n, 1);
  assert.equal(observer.prepare("SELECT count(*) AS n FROM fixture_events").get().n, 1);
});

test("failure rolls back head, edge and event together", (t) => {
  const { writer, observer } = fixture(t);
  assert.throws(() => commit(writer, 7n, "A", "B", { failAfterEdge: true }), /InjectedFailure/);
  assert.equal(readHead(observer), 7n);
  assert.equal(observer.prepare("SELECT count(*) AS n FROM fixture_edges").get().n, 0);
  assert.equal(observer.prepare("SELECT count(*) AS n FROM fixture_events").get().n, 0);
  commit(writer, 7n, "A", "B");
  assert.equal(readHead(observer), 8n);
});

test("wrong installation cannot advance even the right numeric revision", (t) => {
  const { writer } = fixture(t);
  assert.throws(() => commit(writer, 7n, "A", "B", { installation: otherOwner }), /GraphVersionConflict/);
  assert.equal(readHead(writer), 7n);
});

test("schema rejects extra singleton, zero, text and truncated revisions", (t) => {
  const { writer } = fixture(t);
  assert.throws(() => writer.prepare("INSERT INTO graph_topology_head VALUES (2,?,?)").run(owner, bytes(7n)));
  for (const bad of [bytes(0n), Buffer.alloc(7), "00000007", 7]) {
    assert.throws(() => writer.prepare("UPDATE graph_topology_head SET revision=?").run(bad));
    assert.equal(readHead(writer), 7n);
  }
});

test("u64 revisions remain exact above JavaScript and SQLite signed ranges", (t) => {
  const { writer, observer } = fixture(t);
  const large = (1n << 63n) + 17n;
  writer.prepare("UPDATE graph_topology_head SET revision=?").run(bytes(large));
  commit(writer, large, "A", "B");
  assert.equal(readHead(observer), large + 1n);
  writer.prepare("UPDATE graph_topology_head SET revision=?").run(bytes((1n << 64n) - 1n));
  assert.throws(() => commit(writer, (1n << 64n) - 1n, "B", "C"), RangeError);
  assert.equal(readHead(observer), (1n << 64n) - 1n);
  assert.equal(observer.prepare("SELECT count(*) AS n FROM fixture_events").get().n, 1);
});
