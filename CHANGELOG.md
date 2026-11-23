# Changelog

All notable changes to trumly. The format is before 1.0 (see the README): changes are meant to be additive, and anything that could affect an existing project is listed under **Upgrading**.

## Unreleased

### Upgrading

- **`derived` is now a reserved node type**, like `plane`, `rule` and `type`. A project with its own node type called `derived` will find those nodes exempt from rules, required fields and link-end checks, and listed apart from its content. Rename the type if that is not what you want.
- **The `deployment` pack is removed.** Its types and link types (`region`, `network-zone`, `placed-in`, `in-region`, `connects-to`) are in `hosting`, which a project that used it should add instead.
- Rebuild the wasm (`scripts/build-wasm.sh`) and sync it to the editor and cloud to get derived values there.

- **Derived templates are limited**: at most 32 placeholders, no placeholder that follows more than 5 links, and no value longer than 1024 characters. A template past either is unusable (`missing: ["template"]`), like a malformed one.
- **A pack's `name` must be lowercase letters, digits and single hyphens (48 characters at most)**, as its namespace already had to be. `trumly pack check` also no longer puts a pack's name in the scratch folder it builds samples in.

### Added

- **Naming pack tidy-up.** The `name-pattern` type no longer has `separator` or `body` fields (the pattern text already holds the separator, and every node has a Markdown body), and `slot` on a `name-token` is optional. Patterns in the hosting, api, containers, kubernetes, data-model and dns packs no longer carry `separator`.
- **Pack tooling.** `npm run mint -- <pack>` starts the next version of a pack, `npm run status` lists which versions are locked and which packs are retired, and `scripts/check-examples.mjs` runs every rule over every pack example.

- **Retiring packs.** A pack's `meta.json` can say `deprecated: {since, replaced-by, note}` for the whole pack and `withdrawn: {<version>: reason}` for a defective version. Neither changes a pack's checksum, so they can change after a version is locked. Over MCP, `list_packs` shows both (`allowed` is false for a withdrawn version) and tells a project about a retired pack it has (`info`), `add_pack` refuses a withdrawn version, and refuses a deprecated pack to a project that does not have it (one that has it can upgrade, with a `warnings` sentence). A library folder reads the same `meta.json`. `repo` and `homepage` in a pack's `meta.json` must be `https://` addresses.

- A derived template can end in `@type` to read a type name: `{@type}` is this node's own, `{runs-in/@type}` is the type of each node reached. A field really named `@type` still wins. Before, only `@type` in the middle of a path (the type definition) worked.

- **Text length checks, and rules that read derived values.** `field_is` gains `min_length` and `max_length` (characters) and `max_bytes` (UTF-8 bytes) for text. When a node has no field of the name a `field_is` check names, the check reads the node's derived value of that name, so a rule can hold a calculated name to a limit (`handle: {max_length: 63}`). `field_is` can also name the node's own `id` and `title`. A real field wins; a name that cannot be worked out counts as missing; several values are each checked. This also means a rule's result can now depend on a derived definition: the Rust core and the reference both compute derived values before the rules.
- **Derived values have a field name.** A `derived` definition keeps its own `name` and can say what the value is called on a node with `field-name`, so `autoscaler-handle` and `namespace-handle` can both give a node's `handle`. Without it the `name` is the value's name, as before.
- **Starter patterns.** A pack's manifest can list `starter-patterns`: the patterns offered with the editor's "Include starter patterns" tick when the pack is added. Without it the tick offers the `default-pattern` alone, as before. A name the pack lacks is a lint error. Over MCP, `list_packs` and `add_pack` give `starter_patterns`, and `list_patterns` marks `starter`.
- **Pack-qualified names.** Two packs may now define a type or link type with the same name. A node, link or rule says which with `<namespace>:<name>` (`k8s:service`); a bare name still works where one definition has it. Inside a pack a bare name means the pack's own definition, else the nearest pack it requires, else the project's; two packs at the same distance leave it ambiguous (`ambiguous_type`, `ambiguous_link_type`, warnings; nothing fails). Spec section 4a. The pack lint no longer reports a clash between unrelated packs, adding a pack no longer refuses a name another pack defines (your own definition still clashes), `describe_types` lists a shared name per pack, and the editor's type picker shows the pack beside it.
- **Packs can ship derived values.** `derived` is now a definition kind a pack may carry, so a calculation that is a fact about the pack's own types (an endpoint's `url`, an image's `reference`, an environment's `handle`) is there as soon as the pack is added, not only after a pattern is lifted. The pack lint checks the type in `applies_to` and the links a template follows. The api, containers, naming and kubernetes packs now ship theirs, and the patterns that only carried those values are gone.
- **Optional packs ("works with").** A pack's manifest can list `optional` packs: ones it works with when the project has them. They are never added for you and never required, but one that is present counts for name resolution at the same distance as a requirement (spec 4a). `list_packs` returns `optional`; `add_pack` returns `works_with`, the optional packs the project lacks. Examples and patterns take an `optional` list too: a pattern's nodes that need one are left out until it is added, and the editor says so. The core now follows `describes` and `stands-for` by their qualified spelling as well (`conv:describes`).
- **Derived values** (spec section 7a). A node of type `derived` defines a calculated name from a template over a node's fields and links, for example `{@type/^stands-for/token}-{workload}-{runs-in/token}-{instance}`. Placeholders follow links out (`{link/field}`), in (`{^link/field}`) and to the node's type definition (`{@type/…}`). A placeholder that reaches several nodes gives several values. The most specific definition for a name wins. The values appear as `derived` on nodes in `check --json` and `get_node`, are never written to the files, and cannot be read by rules. Missing pieces are reported, not failed.
- **Overrides.** A node that is an exception can say what its name really is with `overrides: {<name>: "value"}`. It replaces the template's value and keeps what the template gave as `computed`. Both `search` (MCP) and the editor find a node by its override.
- **Search by calculated name.** `search` matches derived values; the editor's node table, phone list and link-target picker do too. The nodes table can show a calculated name as a column from its Columns menu (off by default), and the map can label nodes by one (Display > Node labels).
- **Derived values in the editor**: a Derived kind in Definitions, a "Calculated names" card on each node with an override box, and a Try it preview on each definition.
- **Conventions reach agents.** `describe_types` returns `conventions` for each type: what describes the type or its plane (naming rules, tokens), each with the nodes that link to it, then any node that `stands-for` the type. The new-node form shows them as "How this is written".
- **Patterns across packs.** Lifted nodes record `from-pattern: <pack>/<name>`. A node whose links need a pack the project lacks is left out (with what hangs from it) and the preview says which pack it needs. Adding the same pattern again adds only what is missing, and adding a pack says when a lifted pattern has more nodes that now fit. Over MCP: `get_pattern` gives `record_origin`, `list_patterns` returns `lifted`, and `add_pack` returns `patterns_ready` (only what the packs just added made possible; `fits_ids` names the nodes). The hosted MCP now does the same.
- **One pack, many instances.** A pack can name a `default-pattern` in its manifest (a folder under `patterns/`), the starter to lift as another one: a flow per scenario, a lifecycle per thing. The editor offers "Add another" (in the Add menu and on the pack's row), asks for a title first, and takes a fresh id prefix each time. Over MCP, `list_packs` gives `default_pattern`, `list_patterns` marks `default`, and `add_pack` returns `default_patterns`. A `default-pattern` that names a pattern the pack lacks is a lint error.
- **Links to a type.** A link's target can be a node or a type definition ("every node of that type"), chosen in the link row, including on a phone.
- `get_index` lists derived-value definitions in their own section.
- **Cloud:** derived values are cached, refilled after saves and checked against the project version, and `GET /api/derived?q=…&partial=1` to look a node up by value. Definitions do not count toward the plan's node cap.

### Changed

- **`@type` works at any hop of a derived-value placeholder**, not only the first: `{details/@type/^stands-for/value}` reaches the token for the type of the node a link points at. Placeholders that used it in a later position reached nothing before, so this only turns a missing value into a value. New fixture `derived-type-mid-hop`.

- The editor's field filter is one three-part control (field, "is", value) that stays compact when the filter panel is folded.
- The Definitions toolbar drops its counts and then folds its kind tabs into a dropdown before it wraps, and grows back when there is room.

### Fixed

- The "new version" banner no longer reappears or loops after a release: the app version is now the deployed commit, not a per-build timestamp.
