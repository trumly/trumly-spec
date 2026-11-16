# trumly quickstart

trumly keeps a design as a folder of Markdown files (one node per file) that you can read, diff and commit. The `trumly` command checks the project and changes it, and `trumly-mcp` lets an AI agent work on it.

Run `trumly quickstart` any time to print this guide, and `trumly help` for every command.

**Status: version 0.1, before 1.0.** The format is still growing. We expect changes to be additive (new fields, node types and checks that older projects simply ignore, and readers ignore what they do not know). A change that would break an existing project is called out in the release notes with how to migrate.

## What you need

- A terminal on macOS or Linux (Windows is untested).
- `git`, to clone the repository.
- Rust 1.85 or newer with `cargo`, from [rustup.rs](https://rustup.rs). `rustup update` brings an older one up to date.
- A C compiler and linker for Rust to use: the Xcode command line tools on macOS (`xcode-select --install`), `build-essential` or your distribution's equivalent on Linux.
- A few minutes for the first build, and a network connection for it. After that trumly works offline, apart from fetching packs that are not built in.
- Optional, for step 5: an AI client that supports MCP servers, such as Claude Code (the one we have tested), Cursor or VS Code.

You do not need an account. The core packs are built into the program.

## 1. Install

trumly is a Rust program. During the beta you get it by cloning the repository and building it with `cargo`. Ready-made downloads for each platform will follow once we have tested them. Cloning also gives you the worked example, the packs and the format spec.

    git clone https://github.com/trumly/trumly-spec
    cd trumly-spec
    cargo install --locked --path crates/cli     # the trumly command
    cargo install --locked --path crates/mcp     # trumly-mcp, the server for AI agents
    trumly help                                  # check it works

That puts `trumly` and `trumly-mcp` in `~/.cargo/bin`, which `rustup` normally adds to your path. To update, `git pull` and run the two `cargo install` lines again.

`--locked` builds the exact dependency versions we tested, and we run `cargo audit` on every change. To report a vulnerability, see [`SECURITY.md`](https://github.com/trumly/trumly-spec/blob/main/SECURITY.md).

## 2. Try the worked example

`examples/ops-api` is the design of a small ops status API, from goal to decision. It is the same project as the demo on the website. Copy it into a `trumly` folder in your home folder (where your own projects will go too), add the core definitions, and look around.

Start in the `trumly-spec` folder you cloned in step 1. If you are still there from step 1, you are in the right place; `pwd` should end in `trumly-spec`.

    mkdir -p ~/trumly/ops-api
    cp -R examples/ops-api/. ~/trumly/ops-api/    # a copy in ~/trumly, so the repository stays untouched
    cd ~/trumly/ops-api
    trumly pack add core
    trumly check
    trumly show goal-timely-visibility

`check` prints the mark and what it looked at, then one row per rule (`pass`, `warn` or `FAIL`), and ends with a summary line. For this example it is `0 failing · 2 warnings · 41 passing   exit 0`; the two warnings are the decisions that do not yet weigh any options. The exit code is 1 when a rule fails, so you can run it in CI.

Colour is only decoration: it is off for `NO_COLOR`, `--no-color` and anything that is not a terminal, and the words say the same either way.

## 3. Start a project

Make your projects beside the worked example in `~/trumly`, not inside `trumly-spec` or the worked example: the checker reads every Markdown file below the folder it is given, so a project inside another one is read as part of it.

The quickest way is `trumly init`: it asks a few short questions (what it is called, what it is for, who reads it, who owns it, where you are) and sets the folder up with the core pack and an overview node. Pass the answers as flags (`--name`, `--purpose`, `--audience`, `--owner`, `--approach`) to skip the questions. It also asks once whether you want to share anonymous usage data (step 6).

    mkdir -p ~/trumly/my-design && cd ~/trumly/my-design
    trumly init

Or do it by hand. A project is any folder, and the core is one pack:

    mkdir -p ~/trumly/my-design && cd ~/trumly/my-design
    trumly pack add core             # goals, requirements, components, plans and decisions, with their rules
    trumly check                     # validate and run the rules (exit 1 when one fails)

The core is six packs in one bundle: `links` (the links they share), `about` (what the project is for), `define` (goals, requirements, acceptance criteria), `design` (components, datastores, interfaces), `plan` (teams, increments, work packages) and `governance` (decisions and principles). Add detail for a technology or discipline with an enrichment pack when you need it:

    trumly pack list                 # what is available, and what this folder already has
    trumly pack add data-model       # also: api, deployment, dns, hosting, resilience, runtime

A pack brings what it requires. If its names clash with definitions you already have, `pack add` stops and tells you; choose `--use-existing` or `--replace-existing`.

## 4. Everyday commands

    trumly check [DIR]               # errors, warnings and rule results
    trumly show NODE-ID              # one node: links in and out, rule results
    trumly index --write             # a Markdown map of the whole project (.trumly/INDEX.md)
    trumly rename node OLD NEW       # links, parents and rule filters follow
    trumly rename type OLD NEW       # a type, everywhere it is used
    trumly rename link-type OLD NEW
    trumly pack status               # packs in the project, and what you changed in them
    trumly pack remove NAME
    trumly pack detach NAME          # let go of a pack but keep its definitions as your own

Every command that takes `DIR` defaults to the current folder. Commit the folder to git; the files are the source of truth.

## 5. Let an agent work on it

`trumly-mcp` is an MCP server over a project folder. Your AI client starts it and talks to it; you do not run it yourself. It is read-only unless you add `--write`, which lets the agent change the files in the folder (nothing is committed, so commit first and review with `git diff`).

Find where it was installed, and the full path of your project:

    which trumly-mcp
    cd ~/trumly/my-design && pwd

**Claude Code**: from any folder,

    claude mcp add trumly -- trumly-mcp --write /full/path/to/my-design

Start a new Claude Code session (in the project folder if you like) and ask it to use trumly. `claude mcp list` shows whether the server connected. Drop `--write` to let the agent read only.

**Other clients** that run local MCP servers (Cursor, Windsurf, VS Code, Zed, Cline, Codex and Gemini CLI, among others) need a server entry with the same command and arguments. Most use this JSON; check your client's MCP settings for where it goes:

    {
      "mcpServers": {
        "trumly": {
          "command": "/full/path/to/trumly-mcp",
          "args": ["--write", "/full/path/to/my-design"]
        }
      }
    }

Use the full path to `trumly-mcp` (from `which`): apps started from the desktop often do not have `~/.cargo/bin` on their path. Any client that can run a stdio MCP server works; the server speaks JSON-RPC on stdin and stdout.

Chat apps in a browser or on a phone cannot run a program on your computer. They need a hosted server, which is what trumly cloud provides.

Then ask the agent something that makes it use the project:

> Use the trumly tools. Read the index, run the check, and tell me what this design is missing.

It should call `get_index` first, then `get_node`, `list_nodes` and `check`; with `--write` it can add and change nodes, and each write returns what the checker now says. `spec/MCP.md` lists every tool.

## 6. Usage data

Off by default. `trumly init` asks once whether you will turn it on, and you can change your mind at any time: it helps us grow, and proves that people are using trumly. `trumly telemetry status` shows where it stands, `trumly telemetry on` turns it on, `trumly telemetry show` prints exactly what is kept (anonymous counts, never your content), and `trumly telemetry off` turns it off and deletes it. `DO_NOT_TRACK=1` always wins.

## Where next

- `spec/FORMAT.md`: the file format and rule semantics
- `spec/MCP.md`: every tool an agent can call
- The [hosted demo](https://trumly.co/embed/index.html?model=ops-api&base=/embed/models/) shows the same example in a visual editor, in your browser.
