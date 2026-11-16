// Proves a trumly.wasm build answers the agent tools the way the Rust core does.
//   node crates/wasm/js/verify-tools.mjs dist/trumly.wasm
import { readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { loadTrumly } from "./trumly.mjs";

const dir = join(dirname(fileURLToPath(import.meta.url)), "../../../fixtures/tools-cases");

/** `want` is a subset of `got`: objects need every key given, lists the same length with each item a subset. */
export function subset(got, want, at = "result") {
  if (Array.isArray(want)) {
    assert.ok(Array.isArray(got) && got.length === want.length, `${at}: expected ${want.length} items, got ${JSON.stringify(got)}`);
    want.forEach((w, i) => subset(got[i], w, `${at}[${i}]`));
  } else if (want && typeof want === "object") {
    assert.ok(got && typeof got === "object", `${at}: expected an object, got ${JSON.stringify(got)}`);
    for (const [k, w] of Object.entries(want)) subset(got[k] ?? null, w, `${at}.${k}`);
  } else assert.deepEqual(got, want, `${at}: expected ${JSON.stringify(want)}, got ${JSON.stringify(got)}`);
}

/** Run every case through `tool(files, name, args)`, which returns the result or throws the tool's error. */
export function runAll(tool) {
  let n = 0;
  for (const f of readdirSync(dir).filter(f => f.endsWith(".json")).sort()) {
    const c = JSON.parse(readFileSync(join(dir, f), "utf8"));
    const files = Object.entries(c.files);
    c.calls.forEach((call, i) => {
      const label = `${f} call ${i} (${call.tool})`;
      let got, err;
      try { got = tool(files, call.tool, call.args); } catch (e) { err = e.message; }
      if (call.error) assert.ok(err?.includes(call.error), `${label}: error '${err}' does not contain '${call.error}'`);
      else {
        assert.equal(err, undefined, `${label}: ${err}`);
        if (call.text) for (const t of call.text) assert.ok(String(got).includes(t), `${label}: text lacks '${t}'`);
        else subset(got, call.expect, label);
      }
      n++;
    });
  }
  return n;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const trumly = await loadTrumly(readFileSync(process.argv[2]));
  console.log(`${runAll((files, tool, args) => trumly.tool(files, tool, args))} tool calls match.`);
}
