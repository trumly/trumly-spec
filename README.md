<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/brand/trumly-spec-lockup-reversed.svg">
    <img alt="trumly spec" src="docs/brand/trumly-spec-lockup.svg" height="56">
  </picture>
</p>

The file format, rules language, reference core, command line, MCP server and core packs for trumly: a system's design, kept as a typed graph in git. Open source (Apache-2.0).

**Status: version 0.1, before 1.0.** The format is still growing. We expect changes to be additive (new fields, node types and checks that older projects simply ignore, and readers ignore what they do not know). A change that would break an existing project is called out in the release notes with how to migrate.

    trumly quickstart      # a short guide to getting going
    trumly help            # every command

- `docs/QUICKSTART.md`: install, a worked example, starting a project and connecting an agent (also `trumly quickstart`)
- `spec/FORMAT.md`: the file format and rule semantics
- `crates/core`: the engine (pure; no file or network access)
- `crates/fs`: reads a project folder from disk
- `crates/cli`: the `trumly` command
- `crates/mcp`: `trumly-mcp`, an MCP server so agents can read, query and check a project
- `crates/wasm`: the engine as `trumly.wasm` for the browser (plus `js/` glue)
- `reference/`: a small Python reference of the core behaviour and a converter, used to cross-check the Rust
- `fixtures/cases/`: conformance suite (inputs plus `expected.json`); every implementation must match it
- `examples/ops-api`: the worked example (an ops status API, from goal to decision); the demo on the site is the same project. The core packs carry their own small examples in `packs/<name>/examples`.
- `attic/`: code kept for future use, not built or shipped

    cargo test                                  # unit tests plus the full conformance suite
    cargo run -p trumly -- init /tmp/my-design     # asks a few questions and sets a project up
    cargo run -p trumly -- check examples/ops-api
    cargo run -p trumly -- show goal-timely-visibility examples/ops-api
    cargo run -p trumly -- index examples/ops-api  # a Markdown map of the project; --write saves .trumly/INDEX.md
    ./scripts/build-wasm.sh                     # needs rustup and node
    ./scripts/build-docs.sh                     # API docs (rustdoc); pass a folder to choose where they go

Use the MCP server from an agent by pointing it at `target/release/trumly-mcp <project folder>`. It is read-only unless you start it with `--write`, which also lets the agent create, change, rename and delete nodes in the working copy (you review with `git diff`). Tools: `get_index` first, then `get_node`, `list_nodes`, `search`, `get_links`, `check`, `status`, `list_packs`, and the examples and patterns of the packs the project has; with `--write`, `put_node`, `delete_node`, `rename_node`, `add_pack`, `remove_pack` (community packs only with `--allow-community`). The contract is in `spec/MCP.md`.

Regenerating fixtures: `python3 fixtures/tools/make_cases.py && python3 fixtures/tools/bless.py`, then read the diff.

## Where packs come from

The core bundle (`core`, `links`, `about`, `define`, `design`, `plan`, `governance`) is built into the binary and is the only thing that is. `trumly pack` also searches `--packs DIR` (or `$TRUMLY_PACKS`, else `./packs`), `~/.trumly/packs` (yours) and `~/.trumly/community` (other people's, from publishers we have not verified). The first place to have a pack's name wins. `trumly pack add NAME` copies a pack into the project together with what it requires and includes (`--only a,b` picks children of a collection), and stops without writing anything on a name clash.

## Usage data (optional, off by default)

Nothing is recorded until you opt in (`trumly init` asks once, or run `trumly telemetry on`). `trumly telemetry off` stops recording and deletes the install id and the queue; `DO_NOT_TRACK=1` always wins.

- **What is kept:** a random install id, the version, your operating system, the name of what was done (for example `pack_added`) and rounded counts. Never node ids, titles, field values, file paths, project names or any project content.
- **Where it goes:** events are queued on your computer first (`trumly telemetry show` prints exactly what is waiting). `trumly telemetry send` sends them to `https://telemetry.trumly.co/v1/events`. Set `TRUMLY_TELEMETRY_URL` to your own collector to send them there instead, or to `off` to keep them on your computer. If the endpoint is down they are sent later.
- **Logs:** the host of `telemetry.trumly.co` keeps ordinary request logs, which can include your IP address, for its own operations. trumly does not read or store them.

## Scratch planes

A `plane` node with `fields.scratch: true` is a sandbox for drafting (see section 4 of `spec/FORMAT.md`): rules skip its nodes and nothing may link to or from it, or parent across its edge, until its nodes are promoted onto a real plane by changing their `plane`.

## Contributing

See `CONTRIBUTING.md`. In short: issues are welcome; contributing code and publishing your own packs are not open yet, and how they will work is not decided ([what is not decided yet](https://spec.trumly.co/about#not-decided)).

© 2026 Catchlove Ltd. Licensed under the Apache License 2.0 (see `LICENSE` and `NOTICE`).
