// Proves a trumly.wasm build agrees with the conformance fixtures.
//   node crates/wasm/js/verify.mjs dist/trumly.wasm
import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { join, relative, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { loadTrumly } from "./trumly.mjs";

const casesDir = join(dirname(fileURLToPath(import.meta.url)), "../../../fixtures/cases");

function readModel(root, dir = root, out = []) {
  for (const name of readdirSync(dir).sort()) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) readModel(root, p, out);
    else if (name.endsWith(".md")) out.push([relative(root, p).split("\\").join("/"), readFileSync(p, "utf8")]);
  }
  return out;
}

/** Run every fixture case through `check` and compare with expected.json. */
export function runAll(check) {
  let n = 0;
  for (const name of readdirSync(casesDir).sort()) {
    const dir = join(casesDir, name);
    if (!existsSync(join(dir, "expected.json"))) continue;
    const expected = JSON.parse(readFileSync(join(dir, "expected.json"), "utf8"));
    const got = check(readModel(join(dir, "nodes")));
    assert.deepEqual(got, expected, name); // key order is irrelevant; 1 and 1.0 are the same number in JS
    n++;
  }
  return n;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const t = await loadTrumly(readFileSync(process.argv[2]));
  console.log(`wasm: ${runAll((files) => t.check(files))} fixture cases match`);
}
