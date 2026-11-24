# Design graph format, v0 (draft)

A project is a folder of Markdown files. One file is one node, with typed links to other nodes. The folder is the source
of truth; any index or database built from it is disposable. Rules written in the project itself check the whole.

## Example

A node is a Markdown file with YAML front matter. This one is the Ops API from the example project:

```markdown
---
id: component-ops-api
type: component
title: Ops API
plane: dsn--design
fields:
  kind: service
  owner: Platform
  status: planned
links:
  - type: satisfies
    to: requirement-list-jobs
  - type: writes
    to: datastore-ops-db
  - type: runs-in
    to: environment-production
---

Serves the list of jobs and their status to operations staff.
```

`satisfies`, `writes` and `runs-in` are link types, and each `to` names another node in the project. `dsn--design` is a plane
that a pack provides (see Packs).

The front matter is the text between a first line of `---` and the next line that is exactly `---`.
The body is everything after, trimmed. A `---` line inside the body is just body.

## 1. Files

The key words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.

- Every `*.md` under the root, at any depth, is a node file. Other files are ignored, and so are notes kept next to a project: any file named exactly `README.md`, and any file or folder whose name starts with a dot.
- Files are read in order of their root-relative path (`/` separators, bytewise sort).
- The filename stem must equal the node `id`. Directories carry no meaning.
- Encoding UTF-8; `\r\n` is read as `\n`.

### Naming (recommended for writers)

Readers accept any id that is a valid file name. Tools that create files SHOULD use ids that work on
every system and cannot collide:

- lower-case ASCII letters, digits, `-` and `_`; start and end with a letter or digit; at most 100 characters;
- not a name Windows reserves (`con`, `prn`, `aux`, `nul`, `com0`-`com9`, `lpt0`-`lpt9`);
- no two ids that differ only by case (they are one file on macOS and Windows).

Existing files keep their names, whatever they are called.

**Namespaces.** `--` (two dashes) is reserved. An id containing it is `<namespace>--<local id>` and belongs to a pack:
`acme--type-actor` is the `type-actor` of the pack whose namespace is `acme`. Tools that create nodes for a person MUST NOT
produce `--`, so a pack can never collide with someone's own ids. Such a node also carries `fields.pack:
<name>@<version>`; the two must agree (see `namespace_mismatch`).

By convention an id is `<type>-<name>` (`component-ops-api`, `requirement-list-jobs`): the prefix is the
node's type, the rest is its name. Sorted listings then cluster by type with no folders. Planes are the
exception, since other nodes name them in `plane:` (`dsn--design`, not `plane-dsn--design`). The convention is not checked;
it is a default for tools to follow when creating a node and when its type changes.

## 2. Node

| key | type | notes |
|---|---|---|
| `id` | non-empty string | required |
| `type` | string | required for a valid project (see `missing_type`) |
| `title` | string | defaults to `id` |
| `parent` | string | id of the containing node; at most one |
| `plane` | string | id of a node of type `plane` |
| `fields` | map | anything; unknown fields are allowed |
| `links` | list | see below |

Unknown top-level keys are ignored. A wrong type for a known key is `invalid_frontmatter`.

**Link**: `type` and `to` (non-empty strings), optional `key` (string, distinguishes parallel links of
one type to one target) and `fields` (map). A link is stored on its source node only. Inbound links are derived.

**Reserved types**: `plane` (`fields.order`), `rule`, `group` (containment only), `derived` (section 7a), and the three definition kinds
below (`type`, `link-type` and `pack`). Reserved nodes are never subject to rules. All other types are free.

**Definition kinds** describe the vocabulary a project uses. They are ordinary nodes, so they are versioned,
reviewed and linked like the rest, but the core does not enforce them: checking never fails because a
node uses a type or link type nobody defined. Tools read them to offer menus, shapes and forms.

### The `type` node

Defines a node type. Its `fields` are:

| key           | meaning                                                                                                          |
| ------------- | ---------------------------------------------------------------------------------------------------------------- |
| `name`        | The type it defines (`component`). Required.                                                                     |
| `description` | What the type is for.                                                                                            |
| `label`       | A name for people. Viewers show it in place of the name; files, rules and the checker keep using the name.       |
| `plane`       | The id of the plane its nodes belong on. Tools use it to place a new node. Nothing enforces it.                  |
| `shape`       | A shape name such as `hexagon`. A viewer decides whether to draw it upright or flat.                             |
| `icon`        | A drawing on the face of the node: the name of one of a viewer's own icons, or SVG path data (see below).        |
| `fields`      | A list of field definitions for nodes of this type (see below).                                                  |

`shape` may carry an old suffix (`2d`, `up` or `3d`). It still reads, and the suffix is ignored.

An `icon` is either the name of an icon the viewer provides, such as `container`, or a path of SVG path data on a 24 by 24
grid, starting with `M` or `m`, drawn as strokes in one colour. A viewer ignores an icon it cannot read.

For example, the `datastore` type from the design pack:

```markdown
---
id: dsn--type-datastore
type: type
title: datastore
fields:
  name: datastore
  plane: dsn--design
  description: 'Use for somewhere data lives between runs: a database, a cache, an object store, a search index or a file area.'
  shape: database
  fields:
    - name: kind
      kind: choice
      required: true
      values: [relational, document, key-value, object, search, cache, file]
    - name: owner
      kind: text
      required: true
  pack: design@1.0.0
---
```

**Field definitions.** Each entry in `fields.fields` is `{name, kind, values?, required?, label?}`:

- `kind` is `text`, `number`, `boolean`, `choice`, `list` or `object`. A `choice` has `values`, the list of choices.
- A `list` has `items`, the kind of each item: `text`, `number`, `boolean`, `choice` (with `values`) or `object`.
- An `object`, and a `list` whose items are `object`, have `fields`: a list of the same definitions for the keys inside. Tools
  support one level of nesting; the core stores any depth. The values are ordinary YAML lists and maps. The core reads these
  definitions only to offer a blank value (`[]`, `{}`) in examples.
- `label` is shown in place of `name`.
- `required: true` (the boolean) makes a missing field a `missing_required_field` warning on every node of that type outside a
  scratch. A required field named `body` is the Markdown body, missing while it is blank. If two `type` nodes define the same
  name, their required fields are combined.

### The `link-type` node

Defines a link type. Its `fields` are:

| key           | meaning                                                                                                    |
| ------------- | ---------------------------------------------------------------------------------------------------------- |
| `name`        | The link's `type` (`implemented-by`). Required.                                                            |
| `description` | What the link means.                                                                                       |
| `label`       | A name for people, as for a type.                                                                          |
| `inverse`     | How the other end reads the link (`implements`).                                                           |
| `from`, `to`  | Lists of node types the link goes between. Empty or absent means any.                                      |

A link that breaks `from` or `to` is a `link_endpoint_mismatch` warning.

For example, the `writes` link type from the design pack, the one `component-ops-api` uses to reach `datastore-ops-db`:

```markdown
---
id: dsn--link-type-writes
type: link-type
title: writes
fields:
  name: writes
  description: Use to say which component changes data in a datastore.
  inverse: written by
  from: [component]
  to: [datastore]
  pack: design@1.0.0
---
```

## 3. Values

Frontmatter values are JSON values: null, boolean, number, string, list, map with string keys.

- **Dates and times** are ISO 8601 strings (`2026-10-19`, `2026-10-19T16:00:00Z`). The YAML timestamp type is not used: such values are read as plain strings.
- **Integers** must fit a signed 64-bit integer. **Floats** must be finite (no `.nan`, `.inf`) and are written with a decimal point and no exponent (`1.5`, not `1e3`).
- **Money and other exact decimals** are strings (`"12.50"`). Binary floats cannot hold them exactly.
- **Equality** (used by rule filters) is JSON equality: `1` equals `1.0`, `true` never equals `1`, and this holds inside lists and maps.
- Anything else (YAML sets, binary, aliases, tags, non-string keys) is `invalid_frontmatter`.

**Reading YAML.** frontmatter is YAML 1.2 core schema (`on`/`no`/`yes` are strings, `0755` is the integer 755, no
timestamps). Integers beyond i64 become floats.

## 4. Packs

A pack is a set of nodes (planes, types, link types, rules, derived fields) that can be added to a project. Every node it provides carries
`fields.pack: <name>@<version>`, which records where the node came from so imported and local content can be told apart and
a pack can be updated. A pack carries definitions only, never ordinary nodes: the definition kinds are `type`, `link-type`,
`rule`, `plane` and `derived`. A pack ships a `derived` definition for a fact about its own types (an endpoint's address, an
image's reference); a scheme a customer might choose differently (how AWS resources are named) is better left to a pattern,
which they opt into. Its id uses the pack's namespace like the others (`api--derived-url`), and it is checked like a rule: the
type in its `applies_to`, and the links its template follows, should be defined by the pack or by the packs it requires or
lists as `optional` (a warning otherwise). The `derived` type itself stays a reserved core type that no pack defines.

A pack is described by a manifest, a node of type `pack`, whose id is `<namespace>--pack`:

- `fields.name` and `fields.version` (a string): required. `description`: optional.
- `namespace`: the short prefix of every id the pack provides. Default: the name.
- `requires`: a list of pack names that must be present. An entry is a name or `name@range` (`^0.2.0`, `~1.2.3`,
  `>=1.0.0 <2.0.0`, an exact version). A short version is a compatible range, so `core@1` is `core@^1.0.0` and `core@1.2` is
  `core@^1.2.0`.
- `optional`: a list of packs this one works with when the project has them, in the same form as `requires`. They are never
  installed by adding this pack and never needed for it to load, so two packs may name each other (`containers` works with
  `kubernetes`, which requires `containers`). Names the pack uses that one of them defines are known to the lint, and resolve
  to it when it is present (section 4a).
- `includes`: a list of child packs the pack bundles. A collection may hold only a manifest.
- `role`: `core` (one of the bundle every project starts from, built into the tools and published only by trumly) or
  `enrichment` (everything else, the default).
- `publisher`: who made the pack and answers for it.
- `use-when`: one sentence, at most 200 characters, on when to reach for the pack. Lists and agents show it.
- `default-pattern`: optional. The name of one of the pack's patterns (a folder under `patterns/`) to lift as "another one" when the pack is used many times in one project: a flow per scenario, a lifecycle per thing with states. Tools offer it as "Add another" and give it a fresh id prefix and title each time. It must name a pattern the pack has.
- `starter-patterns`: optional list. The names of patterns the pack has that tools offer as "Include starter patterns" when someone adds the pack: ready-made starting points. When it is empty, the `default-pattern` alone is offered. Each must name a pattern the pack has.
- `license`: optional. The licence the pack is published under, as an SPDX identifier such as `Apache-2.0`.
- `enriches`: the core packs an enrichment pack adds detail to (`design`, `plan`...). It helps people and agents choose and is
  not a dependency. An enrichment pack `requires` the core (`core@1`) so its definitions have core nodes to attach to.
- `installed`: recorded in a project's copy of the manifest by the tool that adds the pack: for each node the pack provided, a
  hash of its content as provided (the `pack` tag excluded). Tools use it to tell a node you changed from one you did not,
  when upgrading or removing the pack. Readers ignore it.

For example, the manifest of the design pack:

```markdown
---
id: dsn--pack
type: pack
title: Design
fields:
  name: design
  namespace: dsn
  version: 1.0.0
  description: 'How the system is built, with no technology chosen: components, datastores, queues, interfaces, externals and the environments they run in.'
  license: Apache-2.0
  publisher: trumly
  role: core
  use-when: 'When you are deciding how the system is put together: its parts, where data lives and what it talks to.'
  requires: [links@^1.0.0, define@^1.0.0]
---
```

A pack SHOULD still prefix its own type and link type names with its namespace (`data-table`), so that bare names stay unique
without help. Names that two unrelated packs share are allowed; section 4a says how they are told apart.

A tool that checks packs SHOULD report: a role other than `core` or `enrichment`; a `core` pack not published by trumly; an
enrichment pack with no `use-when`, or one that does not require the core; an `enriches` entry that is not a core pack it can
reach; an ordinary node in a pack; a name two unrelated packs both define; and a type a pack names but nothing it relies on
defines. Every pack, core or not, ships at least one example that checks clean.

## 4a. Name resolution

A type or link type name is **bare** (`service`) or **qualified** by a pack namespace, `<namespace>:<name>` (`k8s:service`). The
part before the first `:` is a namespace as in section 4; the rest is the definition's `fields.name`. A definition belongs to the
namespace in its id (`k8s--type-service`); one whose id has no `--` belongs to no namespace and is reached by its bare name only.

A name appears in a node's `type`, a link's `type`, a rule's `applies_to`, `node_type`, `to_type`, `ancestor_type`, link and
step `type` and `types`, a `link-type`'s `from` and `to`, and a derived field's `applies_to` and link hops. All of them resolve
the same way, to the definitions the name can mean:

1. A qualified name means the definitions of that name in that namespace.
2. A bare name means every definition of that name in the project. Inside a pack (a `rule`, `derived` or `link-type` node whose id
   has a namespace) the nearest level that has a definition wins: the pack's own, else those of the packs it `requires` and of
   the packs it lists as `optional` that the project has, else those of the packs they in turn require or work with, and so on
   down the chain. A level is a set; a namespace counts at its nearest level only, and cycles do no harm. An `optional` pack the
   project does not have adds nothing. If no level has one, every definition in the project is meant. A pack's rules therefore
   target its own types first, and containers' `workload` means kubernetes' when that pack is present.
3. A name with no definition stays a plain string and is compared by equality, so a node of an undefined type, or one that an older
   reader does not understand (`k8s:service` with no k8s pack), still works.

Two names denote the same thing when their definitions overlap; when either has none, when the strings are equal. Rules and links
match on that, not on the strings.

A bare name that still has definitions in two or more namespaces after that is ambiguous. A node whose `type` is such a name, or
whose link `type` is, gets `ambiguous_type` or `ambiguous_link_type`; so does a pack's `rule`, `derived` or `link-type` node that
uses one (in `applies_to`, a check, a template's link hops, or `from` / `to`) when two packs at the winning level define it, which
is two packs it requires directly, or two unrelated packs when nothing nearer has it. The finding names the bare name; the fix is to
write the qualified name (`b:widget`). Nothing fails to load: an ambiguous name matches the definitions of every namespace it
names, which is what happens today. Two definitions of one name inside one namespace remain `duplicate_definition`.

## 4b. What a pack folder holds

A pack is a folder. The manifest and the definitions sit at the top, and two folders beside them show how the pack is used:

```
<pack>/
  <ns>--pack.md                  the manifest
  <ns>--<kind>-<name>.md         the definitions: planes, types, link types, rules and derived fields
  patterns/<name>/pattern.md     a pattern: its description, and
  patterns/<name>/<node>.md      the nodes it adds to a project
  examples/<name>/example.md     an example: its description, and
  examples/<name>/<node>.md      a whole project that uses the pack
```

Only the files at the top are the pack's definitions (section 4). The nodes inside `patterns/` and `examples/` are ordinary nodes: they are
not loaded with the pack, are never added to a project by adding it, and carry no `pack` tag.

**Patterns.** A pattern is a small, ready-made piece of a design that someone adds to a project and then makes their own: a flow, a naming
scheme, a starter layer. `pattern.md` is a node with `id: pattern` and `type: pattern`, and these fields:

| Field | |
| --- | --- |
| `title`, `description` | What it is called and what it gives you. |
| `use` | `starter` (copy it and carry on) or `inspiration` (read it through). |
| `requires` | The packs it needs, the owning pack included. |
| `optional` | Packs it also uses when the project has them. Nodes that need one are left out of a project without it. |
| `root` | The id of the node in the folder that the rest hang from. It must be one of the pattern's nodes. |
| `prefix` | The id prefix suggested when the pattern is added. |

The other nodes in the folder are the pattern's content; the ones that belong under the root name it as their `parent`. When a pattern is
added, its nodes get new ids built from the prefix, the links between them follow, and each records where it came from in
`from-pattern: <pack>/<pattern>`, so a tool can tell which patterns a project has used and offer the ones that now have more to add.
A pack's manifest names its `default-pattern` and `starter-patterns` (section 4).

**Examples.** An example shows the pack in use: `example.md` is a node with `id: example` and `type: example`, with `title`, `description`,
`use` (as above) and `requires`, and the rest of the folder is a whole small project. An example, with the packs it requires, MUST check
clean: no rule failing at `error` level, and no load or structure error. Every pack ships at least one. The core packs share one worked
example, a status API for operations staff.

**Published packs.** A registry keeps each version of a pack in its own folder, `packs/<name>/<version>/`, and a version never changes after
it is published: a fix is a new version. What a registry knows about the pack as a whole sits beside the version folders, outside what a
version's checksum covers, so it can change after publishing: `packs/<name>/meta.json` holds the `tier` (`verified` or `community`),
an optional `repo` and `homepage` (https addresses), and the two ways to retire a pack: `deprecated: { since, replaced-by, note }`, which
tells people to use something else (the editor and the MCP servers do not add it to a project that does not have it), and `withdrawn: { <version>: reason }`,
which marks one version as defective. A project that already has a retired pack keeps working. Tests for a pack's rules sit in
`packs/<name>/tests/<rule-id>.json`, and the checksum of every published version in `registry.lock.json` at the root. The
[trumly-packs](https://github.com/trumly/trumly-packs) repository describes all of this and the commands that work with it.

A tool that checks packs SHOULD also report: a pattern whose `root` is not one of its nodes, and a pack with no example.

## 5. Load errors

A file that fails to load is reported and skipped; the rest still load.

| kind | when |
|---|---|
| `invalid_frontmatter` | no frontmatter, unparseable YAML, not a map, a known key of the wrong type, a malformed link |
| `missing_id` | no `id` |
| `id_filename_mismatch` | `id` differs from the filename stem |
| `duplicate_id` | the id was already loaded; the first file in sorted order wins |

## 6. Structure checks

| kind | level | `ref` |
|---|---|---|
| `missing_type` | error | none |
| `missing_parent` | error | the missing id |
| `parent_cycle` | error | none |
| `missing_link_target` | error | `type->to` |
| `duplicate_link` | error | `type->to` or `type->to#key`; one per extra occurrence |
| `scratch_boundary` | error | `type->to` or `parent->id` |
| `undeclared_plane` | warning | the plane id |
| `namespace_mismatch` | warning | none |
| `title_too_long` | warning | the title's length in characters |
| `duplicate_definition` | warning | the name |
| `ambiguous_type` | warning | the bare type name |
| `ambiguous_link_type` | warning | the bare link type name; one per name |
| `missing_required_field` | warning | the field name |
| `link_endpoint_mismatch` | warning | `type->to` |
| `rule_matches_nothing` | warning | none |

Details:

- `missing_parent`: Reported only on the node that names the missing parent.
- `parent_cycle`: Reported on every node that reaches the cycle.
- `scratch_boundary`: The node and its target are not in the same scratch (see Scratch planes below).
- `namespace_mismatch`: An id with `--` and no `fields.pack`; a `fields.pack` with no `--` id; a prefix that is not the pack's namespace; or a `pack` manifest whose id prefix is not its namespace.
- `title_too_long`: A `title` that the file sets, other than the id itself, is longer than 30 characters. A name has to fit a map label, a table cell and a file listing; put the longer text in the body.
- `duplicate_definition`: Two or more `type` nodes, or two or more `link-type` nodes, in the same namespace (or both in none) share the same string `fields.name`.
- `ambiguous_type`, `ambiguous_link_type`: See section 4a. On nodes, only non-reserved nodes outside a scratch are checked; on definitions, only `rule`, `derived` and `link-type` nodes that have a pack namespace.
- `missing_required_field`: The node's type has a `type` definition listing that field with `required: true`, and the node leaves it unset: absent, `null` or `""`. `0` and `false` count as set. One finding per missing field.
- `link_endpoint_mismatch`: The link's type has a `link-type` definition whose non-empty `from` does not list the source node's type, or whose non-empty `to` does not list the target's type. Only links from non-reserved nodes outside a scratch, to nodes that exist, are checked. If several definitions share the name, their lists are combined, and each list is read from the namespace of the definition it is in.
- `rule_matches_nothing`: Reported on the rule: it is enabled, its `applies_to` has an `id` key, and no non-reserved node matches.

**Stacks.** A `plane` node may carry `fields.stack`, a non-empty string naming the stack (tower) it belongs to. Without it a plane belongs to the stack named by its pack (`fields.pack` up to the `@`), else to `main`; a scratch is always in `main` unless it names a stack. `fields.order` ranks planes only against others in the same stack, and a viewer is expected to draw each stack as its own tower. A plane may also carry `fields.stack_at`, `[column, row]` (two whole numbers from 0), where a viewer should put its stack among the others: stacks sit in columns, and `row` orders them top to bottom within a column. Planes of one stack should carry the same value; a viewer reads the first plane that has one, and places stacks that have none wherever it likes. Like `stack`, it is display only. A stack is only a way of grouping planes for display: the checker, rules and links ignore it, and a link may join nodes on planes of different stacks.

**Scratch planes.** A `plane` node with `fields.scratch: true` (the boolean, nothing else) is a sandbox for drafting. Nodes whose `plane` names it are in that scratch. Rules never apply to them (no results, no ghosts), and a link or `parent` between a node in a scratch and any node outside that same scratch, in either direction, is a `scratch_boundary` error. Links and parents inside one scratch are ordinary. Ids stay unique across the whole project, so moving a node out of a scratch (changing its `plane`, and dropping or bringing along whatever it links to) never renames it.

Identity of a link is (type, to, key). A parent chain through a missing node never raises an error of its own.

## 7. Rules

A rule says something about the nodes of a project. Its `applies_to` picks the nodes, its `check` tests each one, and every
node picked gets a result at the rule's severity. A result is `pass`, `fail`, `advisory` (the rule has no check) or
`unsupported` (the check is not one this version understands). A rule that requires a link also yields a ghost for each link
a node is missing.

A rule is a node of type `rule` with `fields`:

- `name`: optional, at most 30 characters: what the rule is called in lists and headings. Kept equal to the node's `title`. When absent the title is the name.
- `statement`: text: what the rule says. It is the rule's own field, not its name.
- `severity`: `error`, `warn` or `info`; default `warn`. Any other value is treated as `warn`.
- `applies_to`: a filter map; absent or empty matches every non-reserved node.
- `check`: optional.
- `enabled`: `false` switches the rule off: it produces no results and no ghosts, as if it were absent. Anything else, or no
  `enabled` at all, leaves it on. (Packs ship rules this way so a project can adopt only some of them.)

For example, a rule from the design pack, which says every component satisfies at least one requirement:

```markdown
---
id: dsn--rule-component-satisfies
type: rule
title: Components serve a need
fields:
  name: Components serve a need
  statement: Every component satisfies at least one requirement
  severity: warn
  applies_to: { type: component }
  check:
    require_link: { type: satisfies, node_type: requirement, min: 1 }
  pack: design@1.0.0
---

**Why:** a part that satisfies no requirement is either missing its requirement or not needed.
```

### Choosing nodes

`applies_to` is a filter map, and a node is picked when every key matches.

**Filter keys**: `type`, `plane`, `id` (one id, or a list of ids; non-strings never match), `under` (matches a strict ancestor, following `parent`), or any other
key, compared against `fields`. Equality is JSON equality: `true` is not `1`, a string is not a number,
and an absent field never matches anything, including `null`. A present field holding `null` matches `null`. A filter key may be a field path (see below): it matches when any place the path reaches holds the value, so `methods[].verb: POST` selects nodes with at least one POST method. A malformed path never matches.

**Any of**: every key's value may be a list, and the key then matches when any one item does: `type: [service, queue]` is a service or a queue,
`tier: [gold, silver]`, `'methods[].verb': [POST, PUT]`. A list never means "equal to this exact array" (`'tags[]': gold` already asks whether a list contains
a value), and an empty list matches nothing. Different keys all have to match, so `{type: [a, b], plane: design}` is (a or b) and on the design plane.
A reader that does not know lists will not match a rule that uses one.

**Field paths.** Wherever a check names a field it may name a path into lists and maps. A name that is itself a top-level field is used as is (so a field called `a.b` still works).
Otherwise a path is names joined by `.`: `endpoint.path` is the `path` key of the map in `endpoint`. A name followed by `[]` stands for each item of that list: `methods[].verb` is the `verb` of every method,
`tags[]` is every tag. A check on a path applies to every place it reaches and all must pass; an empty list reaches nothing, so it passes vacuously (add `min_items` to demand one). A missing step, or `[]` on something that is not a list,
counts as a missing field. A malformed path (`a..b`, `a[0]`, an empty name) is `unsupported`. Paths work in `require_fields`, `field_is`, `compare_fields` and in `applies_to` filters (where "any place" replaces "every place").

### Checks

A `check` is a map; every key present must pass (AND). Each key below may be
malformed or unknown; that yields `unsupported`, never a silent pass. A failure outranks `unsupported`.
Where a value is "one or a list", a single item means a one-item list.

| check                 | passes when                                                         |
| --------------------- | ------------------------------------------------------------------- |
| `require_fields`      | every named field is present                                        |
| `field_is`            | every named field meets its constraints                             |
| `compare_fields`      | a field compares as stated with another field or a value            |
| `require_link`        | the node has the stated number of matching links                    |
| `forbid_link`         | no link matches                                                     |
| `path`                | a chain of links exists from the node                               |
| `acyclic`             | the node cannot reach itself by following the named link types      |
| `require_link_fields` | every link of a type carries the named fields                       |
| `any_of`              | at least one of several checks passes                               |

**Field checks**

- `require_fields: [paths]`: every field present (not null, not `""`; an empty list or map counts as present, use `min_items` for that).
- `field_is: {path: {in?, min?, max?, type?, optional?, min_items?, max_items?, min_length?, max_length?, max_bytes?, unique?}}`: the field must satisfy every given constraint.
  A missing field fails unless `optional: true`. `type` is one of string, number, integer, boolean, list, map, null.
  Numbers compare exactly for integers, as floats otherwise. `min_items` / `max_items` (non-negative integers) bound the length of a list; a value that is not a list fails them.
  `min_length` / `max_length` (non-negative integers) bound the length of text in characters, and `max_bytes` bounds its length in UTF-8 bytes (a limit stated in bytes, such as PostgreSQL's 63, differs from one in characters for accented text); a value that is not text fails them. A name the node has no field of is read from the node's own `id` and `title` (so `title: {max_length: 30}` holds a node's name to a length; a node with no title has its id as the title), then from its derived fields (section 7a), so a rule can check a calculated name the same way: `field_is: {handle: {max_length: 63, optional: true}}`. A real field of that name wins. A name that could not be worked out counts as a missing field (hence `optional: true`). A name that gives several values is checked value by value.
  `unique` needs a list: `true` means no two items are equal (JSON equality); a field path (no `[]`) means no two items hold the same value there. Items lacking that field are not compared. Any other `unique` is `unsupported`.
- `compare_fields: one or a list of {left, op, right | value}`: `op` is `== != < <= > >=`. `left` and `right` are field paths;
  `value` is a literal. A missing field fails. Ordering works on number pairs and string pairs only. `left` may use `[]` (every item is compared); `right` must reach exactly one place or the check is `unsupported`.

**Link checks**

- `require_link: one or a list of {type, dir?, node_type?, node_plane?, node_fields?, same_fields?, ancestor_type?+scope?, with_fields?, min?, max?}`:
  counts links of that type, outgoing (`dir: out`, default) or incoming (`in`), whose other end exists and matches
  `node_type`/`node_plane` (`to_type`/`to_plane` are accepted aliases). `ancestor_type` with `scope: same|other`
  requires the nearest ancestor of that type to be the same (resp. a different, both existing) for the two ends.
  `with_fields` lists link fields that must be present. `node_fields` is a map of fields the other end must hold
  (same equality as filters). `same_fields` lists field names for which the other end must hold the same value as
  this node (both must have the field). Bounds: `min` defaults to 1, or 0 if only `max` is given.
  `min`/`max` must be non-negative integers.
- `forbid_link: one or a list of {type, dir?, node_type?, node_plane?, node_fields?, same_fields?, ancestor_type?+scope?, with_fields?}`:
  the same filters as `require_link`; passes only when no link matches. `min` and `max` are not accepted (that is `unsupported`).
- `path: [steps]`: a non-empty list of `{type, dir?, node_type?, node_plane?, node_fields?}`. Passes if a chain exists from the
  node: the first step is followed from the node, each later step from every node the previous step reached. Links to missing
  nodes never count. An empty list, a non-list, or a malformed step is `unsupported`.
- `acyclic: type or [types]`: fails if the node can reach itself by following outgoing links of that type (a self-link counts).
  Every node on a cycle fails; nodes that merely lead into one pass.
- `require_link_fields: one or a list of {type, dir?, fields}`: every link of that type has all the fields present.

**Combining checks**

- `any_of: [checks]`: a non-empty list of checks, each a full check map (its keys are ANDed as usual). Passes if at least one passes. Lists elsewhere (`require_link: [..]`, `require_fields`) mean "all of these"; this is how to say "one of these". If none passes and any of them is `unsupported`, the whole is `unsupported`, because that one might have passed. An empty list, a non-list, or an item that is not a non-empty map is `unsupported`. Nests. It gives no ghosts.

A `check` of `{}`, or no `check`, is `advisory`. A key that is not listed above is `unsupported`.

### Results and ghosts

**Result** per (node, rule): `{node, rule, severity, status}` with status `pass`, `fail`, `advisory` or `unsupported`.

**Ghost**: one per failing outgoing `require_link` item whose minimum is at least 1 and count is below it:
`{node, rule, link, to_type, plane}`, where `to_type` is null if unspecified and `plane` is `node_plane`/`to_plane`,
else the node's plane (null if none).

`component-ops-api` has a `satisfies` link to `requirement-list-jobs`, so the rule passes for it:

```json
{ "node": "component-ops-api", "rule": "dsn--rule-component-satisfies", "severity": "warn", "status": "pass" }
```

Without that link the result would be `"status": "fail"`, and the project would also report a ghost, the link the node is missing:

```json
{ "node": "component-ops-api", "rule": "dsn--rule-component-satisfies", "link": "satisfies", "to_type": "requirement", "plane": "dsn--design" }
```

## 7a. Derived fields

A **derived field** is a read-only value a node has but nobody writes: it is worked out from a template over the node's own
fields and the nodes it links to. The first use is a calculated name (`psql-ops-prod-uks-001`) built from facts the model already holds. The core
knows nothing about names; it only resolves fields and links, and a pack supplies the templates.

A `derived` node defines one. Its `fields` are:

| key | meaning |
|---|---|
| `name` | The definition's own name (`namespace-handle`). Required, a non-empty string. It is also the name of the value on a node unless `field-name` says otherwise. |
| `field-name` | Optional. What the value is called on a node (`handle`). Several definitions, each for its own kind of node, can share one, and tools list it once. Overrides, search and tables use this name. |
| `template` | The text to build, with `{placeholders}`. Required, a string. |
| `applies_to` | A filter, as for rules (section 7): absent or empty means every non-reserved node. |
| `enabled` | `false` switches the definition off, as for rules. |

A definition without a string `name` and `template` is ignored. Derived nodes are reserved: rules never apply to them, and they have no derived fields of their own. Nodes in a scratch have none either.

**Template.** Text with `{placeholders}`. `\{`, `\}` and `\\` write a literal `{`, `}` and `\`. An unbalanced brace or an empty `{}` makes the whole template unusable (`missing: ["template"]`, below), as does an empty template, one with more than 32 placeholders, one with a placeholder that follows more than 5 links, or one that would build a value longer than 1024 characters.

**Placeholder.** Split on `/`. Every segment but the last is a hop through the graph, and the last is the field to read:

- `{workload}`: no slash, so a field of the node itself. A field path is read as in rules (`a.b`, `list[].x`). `id`, `title` and `@type` (the node's type name, as written) read the node's own when no field has that name.
- `{runs-in/token}`: follow links of type `runs-in` out of the node, then read `token` from each node reached.
- `{^used-in/token}`: a `^` before a link type follows links of that type *into* the node.
- `{runs-in/@type}`: `@type` as the last segment is not a hop but a read: the type name of each node reached (`{@type}` alone is this node's own). Used as a hop, with something after it, it is the type definition, as follows.
- `{@type/^stands-for/token}`: `@type` is the type definition of each node reached so far (the first `type` node by id whose `name` is that node's type). Here it leads to the tokens that stand for the node's type. It can come at any hop, so `{details/@type/^stands-for/token}` reaches the token for the type of the node a `details` link points at. A node whose type has no definition reaches nothing.

Hops can be chained. The nodes reached are taken once each, in id order; a link to a node that does not exist reaches nothing. Only strings, numbers and booleans are read, as text: strings as they are, numbers as JSON writes them, booleans as `true` and `false`. A list, a map, `null` or an absent field gives nothing. A malformed placeholder (an empty segment) reaches nothing.

**Values.** A placeholder that reaches several values gives several, and the template is built once for each combination, the first placeholder varying slowest. Repeats are dropped, and at most 64 values are kept.

**Which definition.** For one node and one value name (`field-name`, else `name`), the definitions that apply are compared by what their `applies_to` names: `id` (most specific), then `type`, then `plane`, then anything else or nothing. The most specific wins; a tie goes to the lower definition id. Definitions with different field names (or, without one, different names) do not compete: each gives its own value.

**Result.** A node that any definition applies to gets, for each name, either `{"values": [...]}` (every placeholder reached something) or `{"missing": [...]}`, listing the placeholders, as written between the braces, that reached nothing (`["template"]` for an unusable template). Missing is not a finding: nothing fails and nothing is reported. A model with no `derived` nodes has no derived fields, and a reader that does not know them ignores them.

**Overrides.** A node that is an exception says what its value really is, in an ordinary field: `overrides: {<name>: "value"}`, or a list of values. For a `name` a definition applies to, non-empty string values there replace the template's, in the order written, repeats dropped, at most 64. The result then also has `"override": true` and `"computed"`, the values the template gave (an empty list if it could not be resolved): `{"values": ["legacy-psql"], "override": true, "computed": ["psql-ops-prod-uks-001"]}`. Anything else in `overrides` is ignored: a value that is not a string, a name no definition applies to, or an `overrides` that is not a map. An override gives a node a derived field only where a definition applies, so a search for a name finds the node by the real value, not the one the template would have made.

Derived fields are computed, never stored: they are not written to a node's file, so they cannot go stale. A rule reads one through `field_is` (section 7), where a name the node has no field of is looked up here, an override standing in for the template's value; nothing else in a rule reads them.

## 8. `check --json`

```
{ load_errors: [{file, kind}],
  nodes: [full node, sorted by id; includes body; a node with derived fields also has `derived`, section 7a],
  structure: {errors: [{node, kind, ref}], warnings: [...]},
  results: [...], ghosts: [...] }
```

Everything is sorted: load errors by (file, kind), structure by (node, kind, ref), results and ghosts by
(node, rule), nodes by id. Strings sort bytewise. Object keys are emitted sorted; indentation is not significant
(compare parsed JSON). Exit code is 1 if there is any load error, structural error, or failed `error`-severity result.

## 9. Portability

YAML 1.1 and 1.2 disagree (`no`, `yes`, `on`, octal-looking numbers, exponent floats). The reference implementation reads YAML 1.2, so the conformance cases avoid such bare scalars. Authors should quote them.

## 10. Conformance

An implementation is correct when its `check --json` output equals the `expected.json` of every conformance case in
`fixtures/cases`, for the project that case holds. Each check and finding described here has a case that shows its expected
output.

**Forward compatibility.** A reader ignores top-level keys, fields, node types and link types it does not know, and loads and checks the rest as usual (the `forward-compat` case). That is what lets the format grow by addition without breaking existing projects.
