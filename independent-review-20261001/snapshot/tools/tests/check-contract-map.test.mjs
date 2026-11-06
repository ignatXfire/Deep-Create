import test from "node:test";
import assert from "node:assert/strict";
import { checkContractMap } from "../check-contract-map.mjs";
function valid() {
  return { schema_version: 1, boundary: "Not a proof", contracts: [{
    id: "14.Example/v1", status: "partial", may_be_persisted_as_canonical: false,
    source: "src/example.rs", symbol: "Example", tests: ["tests/example.rs"],
    implemented: ["value checks"], deferred: ["trusted store"]
  }] };
}
const read = () => "pub struct Example; // fixture, not parsed as Rust";
test("accepts explicit partial traceability", () => assert.deepEqual(checkContractMap(valid(), read), []));
for (const [name, edit] of [
  ["unknown envelope field", map => { map.extra = true; }],
  ["duplicate identity", map => { map.contracts.push({ ...map.contracts[0] }); }],
  ["unreviewed completion", map => { map.contracts[0].status = "complete"; }],
  ["persistence claim", map => { map.contracts[0].may_be_persisted_as_canonical = true; }],
  ["hidden remaining work", map => { map.contracts[0].deferred = []; }],
  ["path escape", map => { map.contracts[0].source = "../outside"; }],
  ["absolute Windows path", map => { map.contracts[0].source = "C:/outside"; }],
  ["missing symbol", map => { map.contracts[0].symbol = "Missing"; }],
  ["unknown contract field", map => { map.contracts[0].trusted = true; }],
]) test(`rejects ${name}`, () => { const map = valid(); edit(map); assert.ok(checkContractMap(map, read).length); });
test("missing source fails closed", () => assert.ok(checkContractMap(valid(), () => { throw new Error("absent"); }).length));
