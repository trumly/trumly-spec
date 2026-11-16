// node scripts/sync-core.mjs [PACKS_REPO] [--check]
// The core bundle (packs/core.pin) is owned by the trumly-packs repository and built into the binary from the copy in packs/.
// Without --check this replaces packs/<name>/ with packs/<name>/<version>/ from the packs repository; with --check it changes
// nothing and exits 1 when the copy has drifted (CI runs it). PACKS_REPO defaults to ../trumly-packs.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.argv.slice(2).find(a => !a.startsWith('--')) ?? join(here, '..', 'trumly-packs'));
const check = process.argv.includes('--check');
const pins = readFileSync(join(here, 'packs', 'core.pin'), 'utf8').split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('#')).map(l => l.split('@'));
/** every file under a folder as relative path -> text (examples and patterns included) */
const tree = (dir, rel = '') => readdirSync(join(dir, rel), { withFileTypes: true }).flatMap(e => e.isDirectory() ? tree(dir, join(rel, e.name)) : [[join(rel, e.name), readFileSync(join(dir, rel, e.name), 'utf8')]]).sort((a, b) => (a[0] < b[0] ? -1 : 1));
let drift = 0;
for (const [name, version] of pins) {
  const from = join(source, 'packs', name, version), to = join(here, 'packs', name);
  if (!existsSync(from)) { console.error(`${name}@${version} is not in ${source}`); drift++; continue; }
  const want = tree(from), have = existsSync(to) ? new Map(tree(to)) : new Map();
  const same = want.length === have.size && want.every(([f, t]) => have.get(f) === t);
  if (same) continue;
  drift++;
  if (check) { console.error(`${name}@${version} differs from the packs repository. Run: node scripts/sync-core.mjs`); continue; }
  rmSync(to, { recursive: true, force: true });
  mkdirSync(to, { recursive: true });
  cpSync(from, to, { recursive: true });
  console.log(`synced ${name}@${version}`);
}
// the worked example (examples/ops-api, which the site runs as its demo) is the core pack's own example: one story everywhere
{
  const ex = join(here, 'examples', 'ops-api'), own = join(here, 'packs', 'core', 'examples', 'ops-api');
  if (existsSync(ex) && existsSync(own)) {
    const mine = new Map(tree(ex)), theirs = tree(own).filter(([f]) => f !== 'example.md');
    if (mine.size !== theirs.length || !theirs.every(([f, t]) => mine.get(f) === t)) {
      console.error('examples/ops-api differs from the core pack example (packs/core/examples/ops-api). They must be the same project: change it in trumly-packs, then copy it here.');
      drift++;
    }
  }
}
// nothing but the pinned packs may sit in packs/ (they are all built into the binary)
for (const d of readdirSync(join(here, 'packs'), { withFileTypes: true }).filter(d => d.isDirectory())) {
  if (!pins.some(([n]) => n === d.name)) { console.error(`packs/${d.name} is not in packs/core.pin, but everything in packs/ is built into the binary.`); drift++; }
}
if (check) { console.log(drift ? `${drift} problem(s): the core bundle has drifted from the packs repository.` : 'The core bundle matches the packs repository.'); process.exit(drift ? 1 : 0); }
console.log(drift ? 'Done. Rebuild and run the tests.' : 'Nothing to sync.');
