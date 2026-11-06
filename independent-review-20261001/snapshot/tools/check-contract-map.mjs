import { readFileSync, existsSync, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const registry = "docs/implementation/contracts/contract-map.json";
const plain = value => value !== null && typeof value === "object" && !Array.isArray(value);
const exact = (value, keys) => plain(value) && Object.keys(value).sort().join() === [...keys].sort().join();
const text = value => typeof value === "string" && value.trim().length > 0;
const list = value => Array.isArray(value) && value.length > 0 && value.every(text);
const safe = value => text(value) && !path.isAbsolute(value) && !value.includes(":")
  && !value.includes("\\") && value.split("/").every(part => part && part !== "." && part !== "..");

// Traceability check only: does not parse Rust, run tests or prove claims true.
export function checkContractMap(map, readSource) {
  const errors = [];
  if (!exact(map, ["schema_version", "boundary", "contracts"]) || map.schema_version !== 1
      || !text(map.boundary) || !Array.isArray(map.contracts) || map.contracts.length === 0) {
    return ["Invalid registry envelope"];
  }
  const ids = new Set();
  for (const item of map.contracts) {
    if (!exact(item, ["id", "status", "may_be_persisted_as_canonical", "source", "symbol", "tests", "implemented", "deferred"])
        || !text(item.id) || !/^[0-9]{2}\.[A-Za-z][A-Za-z0-9]*\/v[1-9][0-9]*$/.test(item.id)) {
      errors.push("Invalid contract entry"); continue;
    }
    if (ids.has(item.id)) errors.push(`${item.id}: duplicate identity`);
    ids.add(item.id);
    // Until C-03 has a reviewed admission route, this registry has NO automatic
    // escape hatch that turns a manually edited status into storage permission.
    if (item.status !== "partial" || item.may_be_persisted_as_canonical !== false) {
      errors.push(`${item.id}: unreviewed canonical-persistence claim`);
    }
    if (!list(item.implemented) || !list(item.deferred)) errors.push(`${item.id}: missing explicit scope or gaps`);
    if (!safe(item.source) || !text(item.symbol) || !list(item.tests) || !item.tests.every(safe)) {
      errors.push(`${item.id}: invalid source/test reference`); continue;
    }
    try {
      if (!readSource(item.source).includes(item.symbol)) errors.push(`${item.id}: source symbol missing`);
      for (const test of item.tests) {
        if (!text(readSource(test))) errors.push(`${item.id}: empty test source`);
      }
    } catch { errors.push(`${item.id}: missing or inaccessible source`); }
  }
  return errors;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const resolvedRoot = realpathSync(root);
  const readSource = name => {
    const file = path.resolve(root, name);
    if (!existsSync(file) || !realpathSync(file).startsWith(resolvedRoot + path.sep)) throw new Error("Outside project or absent");
    return readFileSync(file, "utf8");
  };
  const errors = checkContractMap(JSON.parse(readSource(registry)), readSource);
  if (errors.length) { process.stderr.write(errors.join("\n") + "\n"); process.exitCode = 1; }
  else process.stdout.write("PASS contract traceability; all registered canonical contracts explicitly partial\n");
}
