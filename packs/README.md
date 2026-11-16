# The core packs

Only the **core bundle** is built into the trumly binary, so a project can start with no network and no folder. Every other pack is an *enrichment* pack: it adds detail to the core, and lives in the registry.

The packs repository (`trumly-packs`) is the source of truth for all of them, the core included. This folder holds a pinned copy of the core, listed in `core.pin`, that `crates/fs/build.rs` bakes into the binary:

    node scripts/sync-core.mjs [PATH_TO_TRUMLY_PACKS]          copy the pinned versions here
    node scripts/sync-core.mjs [PATH_TO_TRUMLY_PACKS] --check  fail if this folder has drifted (run it in CI)

To change the core, publish a new version in the packs repository, change the version in `core.pin`, sync, rebuild and run the tests. Never edit a pack in this folder by hand: the next sync would undo it, and the check fails until it is undone.

## What is in the core

`core` is a bundle: adding it adds all six.

| Pack | What it is |
|---|---|
| `links` | The links the others share (`derives-from`, `satisfies`, `verified-by`, `governs`, `depends-on`, `supersedes`, `details`, `part-of`, `about`, `relies-on`, `runs-in`, `exposes`). No types. |
| `about` | What the project is for: one `overview` (id `overview`, with its `stage`) and the `convention`s the project is written by. |
| `define` | Why the system exists and what it must do: goals, constraints, assumptions, actors, capabilities, requirements and their acceptance criteria. |
| `design` | How it is built, with no technology chosen: components, datastores, queues, interfaces, externals, environments. |
| `plan` | How it gets built: teams, increments, work packages and risks. |
| `governance` | Why it is the way it is: decisions, the options weighed, consequences, principles. |

Core packs are technology-neutral. Detail for a particular technology or discipline (APIs, data, hosting, containers, DNS...) is an enrichment pack that `requires: core@1`, names the core pack it `enriches`, and attaches to those nodes with `details`.

## Pack manifests

A pack is a folder of plain Markdown definitions (planes, types, link types and rules) that a project adds with `trumly pack add NAME`. Packs carry no code. A published version is immutable: a change is a new version. The manifest (`<namespace>--pack.md`) says:

| field | meaning |
|---|---|
| `name`, `namespace`, `version` | what it is called, the prefix of every id it provides, and its version |
| `description`, `license`, `publisher` | what it is, the SPDX licence, and who answers for it |
| `role` | `core` (only trumly's packs, the only ones built in) or `enrichment` (the default) |
| `use-when` | one sentence, at most 200 characters, on when to reach for it |
| `enriches` | the core packs an enrichment pack adds detail to (informational) |
| `requires` | packs that must be there, as `name` or `name@range`; `core@1` means `core@^1.0.0` |
| `includes` | child packs a bundle brings with it |

Two tiers say who vouched for a pack in the registry: **verified** (reviewed by trumly; our own packs are verified, and `publisher` tells you who wrote it) and **community** (anyone's, not reviewed beyond the automated checks).

See `spec/FORMAT.md` for the format. `trumly pack check` reports what is wrong with a manifest and demands at least one example per pack, and that every example and pattern checks clean.

## Examples and patterns

Every pack shows itself in use. A pack folder can hold:

    examples/<name>/example.md    what it shows, and the nodes beside it
    patterns/<name>/pattern.md    a fragment to lift into a project, and the nodes beside it

The manifest (`example.md` or `pattern.md`) is a node with `fields`:

| field | meaning |
|---|---|
| `title`, `description` | what it is and what it shows |
| `use` | `starter` (copy it and carry on) or `inspiration` (flick through it); patterns are starters |
| `requires` | packs it needs to check clean, its own pack included |
| `root`, `prefix` | patterns only: the node the rest hangs from, and the id prefix used when it is inserted |

The nodes are ordinary project nodes, not definitions: they are never copied into a project by adding the pack. An example is checked in a project that has the packs it names; a rule that matches nothing only counts against the pack's own rules, so an enrichment pack's example is not asked to exercise the whole core. Agents see the examples and patterns of the packs a project has installed through `list_examples`, `get_example`, `list_patterns` and `get_pattern`.

## Naming

Packs stay independent: they connect through the shared links, never by sharing type names.

- **Types are nouns, link types are verbs** (`decision` / `considers`). A noun and its verb inside one pack is fine.
- **One verb per meaning.** A link that means the same across layers lives in `links`. A link that only makes sense inside one pack is prefixed with that pack's word.
- **Prefix a type when its word is generic in its own domain or likely to clash.** Leave it plain when it is neutral and unique across the library.
- **No near-twins across packs.** `trumly pack check` warns when a type or link type in one pack is within two letters of one in another, and when two unrelated packs use the same name. The registry turns both into errors between verified packs; a community pack that clashes with a verified one is flagged on the community pack.
- **Names are short.** Titles are at most 30 characters; a rule has a short `name` and a separate `statement`.
