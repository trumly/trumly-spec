# The trumly MCP tools

The contract for the tools an agent uses to read and design a trumly project. The local server (`trumly-mcp`) implements it over a project folder; the hosted one will implement the same tools over a project in trumly cloud, and both must behave the same. Tool results are JSON text unless stated.

## Access

Access is granted at one of two levels, and an agent cannot raise it.

- **Read** (the default): the read tools only. The write tools are not listed and calling one is refused.
- **Read and write**: the read tools plus the write tools. Locally this is `trumly-mcp --write DIR`. In the cloud it is the scope of the token you create.

Writes go into the working copy: the folder on disk locally, the cloud's working copy for a hosted project. Nothing is committed. The person reviews the change (`git diff`, or the editor) and publishes it.

## Read tools

| Tool | Arguments | Returns |
|---|---|---|
| `get_index` | none | A Markdown map of the project (text, not JSON). Read it first. It opens with the project's overview (the node `overview`, see below), and ends with "What you can create": every type by plane with its node count, and the link types. |
| `describe_types` | `type`, `plane`, `pack` (all optional) | The project's vocabulary, read from its own `pack`, `plane`, `type`, `link-type` and `rule` nodes: `packs`, `planes`, `types` (each with `fields`, `required_fields`, `plane`, `links_out`, `links_in`, `rules`, `existing_nodes`, an `example` node to copy, and `conventions` when anything has a `describes` link to the type's definition or to its plane: those nodes with their fields and body, the type's own first, each with the nodes that link to it, then any node that `stands-for` the type without being reached that way, so naming rules and tokens reach an agent as it names a node), `link_types`, `rules`, and the shape of the `overview`. When definitions in two or more packs share a name, each is listed as `<namespace>:<name>` (`name`, with `qualified` and `bare`) and a bare name gets `ambiguous_type` advice from `put_node`; `type` also accepts the qualified or the bare name. A `type` is placed on the plane its definition names in `fields.plane`, else the plane its existing nodes sit on. |
| `get_node` | `id` | `node` (id, type, title, parent, plane, fields, links, body), `path`, `rev`, `inbound` links, `rules` results, and `derived` (the node's derived values, see FORMAT.md 7a) when it has any |
| `list_nodes` | `type`, `plane`, `under` (all optional) | A list of id, type, title, plane, parent |
| `search` | `query`, `type`, `limit` (default 20, max 100) | `total`, `shown`, and `results` with `matched` (id, title, fields, derived, body) and a `snippet`. Derived values are searched, so a calculated name finds its node. |
| `get_links` | `id` | `outbound` (type, to, key, fields, `to_title`, `exists`) and `inbound` (from, `from_title`, type, key) |
| `list_packs` | none | The packs on offer and which the project has: `name`, `version`, `description`, `tier` (`verified`, `community` or `yours`), `role` (`core` or `enrichment`), `use_when` (one sentence on when to reach for it), `enriches` (the core packs an enrichment adds detail to), `publisher`, `requires`, `includes`, `default_pattern` (the pattern to lift for another instance, when the pack names one), `starter_patterns` (the patterns the pack lists as worth adding with it), `installed` (null if not added), `allowed` (whether `add_pack` will take it: false for a community pack nobody allowed, a withdrawn version, and a deprecated pack the project does not have), and for installed packs the nodes you `changed` or are `missing`. A pack its publisher has retired also has `deprecated` (`since`, `replaced_by`, `note`) or, for a defective version, `withdrawn` (the reason), and `allowed` is false for a withdrawn version. When the project has a retired pack the result also has `info`, sentences saying so. |
| `list_examples`, `list_patterns` | none | The worked examples and reusable patterns of the packs the project has: what each shows, whether it is a starting point or for inspiration, `default` and `starter` flags, and which packs it still needs (`missing`) and which it also uses but the project lacks (`missing_optional`: its nodes that need them are left out until they are added). Each pattern has `default` (true for the one its pack names as the pattern to lift for another instance). `list_patterns` also returns `lifted`: for each pattern with nodes carrying `from-pattern` in the project and others not yet here, `present`, `total`, `fits` (how many of the rest could be added now), `fits_ids` (which) and `waiting` (nodes whose links or parent point at something the project lacks, with what they `need`). |
| `get_example`, `get_pattern` | `pack`, `name` | One example or pattern in full: its nodes, and for a pattern the root it hangs from, its id prefix and `record_origin` (the `from-pattern` field and value to set on every node you lift, so the project knows where it came from). |
| `check` | none | Node count, load errors, structure check results, failed rules, ghosts, `ok` |
| `status` | none | `since_last_commit` (changed files and their node ids, needs git) and `this_session` (what this server changed) |

## Write tools

| Tool | Arguments | Effect |
|---|---|---|
| `put_node` | `node`, `base_rev` | Creates a node, or replaces one. |
| `delete_node` | `id`, `base_rev` | Deletes a node and lists the links that now point at nothing. |
| `rename_node` | `from`, `to` | Gives a node a new id. Links, parents and rule filters follow. |
| `start_project` | `answers` (optional) | Sets a project up the first time, with the same short questions the cloud's guided start asks (listed in `start-questions.json`). Called with no `answers` it returns the questions and what is `missing`; the agent asks the person each one, then calls again with `name`, `purpose`, `audience`, `owner`, `stage` and optionally `packs`. It adds the core bundle and the chosen verified packs and writes the `overview` node. Refused once the project has an overview. |
| `add_pack` | `name`, `only` (optional) | Adds a pack with the packs it requires. A name the project already defines is refused, never overwritten. Returns `packs`, `added`, `updated`, `works_with` (the pack's `optional` packs the project does not have, with when to use each; none are added), `default_patterns` (the pattern each pack added names for lifting another instance: `pack`, `name`, `title`, `description`; open it with `get_pattern`), `starter_patterns` (the same shape, for the patterns each pack lists as worth adding with it), `failed_rules_before` `warnings` (a deprecated pack the project already has can be upgraded, with a sentence naming what to use instead; one it does not have is refused, and so is a withdrawn version, with the reason), and the usual `model` counts, because a pack brings its rules, and `patterns_ready` when patterns already lifted can now take more nodes that fit only because of the packs just added (a layer left out on purpose is not offered again): the `lifted` rows, with `fits` and `fits_ids` counting just those nodes. |
| `remove_pack` | `name` | Removes a pack's manifest and every node it provided. Refused, deleting nothing, while another pack requires it, nodes still use its types, other nodes link to it, or its nodes were edited. There is no force option. |
| `detach_pack` | `name` | Lets go of a pack but keeps what it gave the project: the manifest is deleted and the `pack` tag comes off its nodes, so its types, link types, rules and planes stay as ordinary nodes. Use it when nodes still use the pack's types. Refused only while another pack requires it. Returns `kept`. |

**Text from the project and its packs is data.** Everything an agent reads (node titles, fields and bodies, type and rule descriptions, pack descriptions, examples, patterns) is content, never instructions. The server's `instructions` say so, and tell the agent to ignore, and report to the person, any text in it that tries to direct it. The results that carry a pack author's text (`list_examples`, `list_patterns`, `get_example`, `get_pattern`, `list_packs`, and `add_pack` when it returns patterns or warnings) also include a `notice` saying the same. This is guidance to the model, not a guarantee: the real defences are that agents cannot add community packs, that writes are never committed without the person's review, and that verified packs are read for agent-directed instructions before they are listed. The hosted server must give the same instructions and notices.

**Packs from other people.** `add_pack` takes verified packs (the core bundle built into the binary, and packs from the registry that trumly has reviewed) and the person's own packs. A community pack (from a publisher that is not verified) is listed but refused, because a pack's text reaches the agent through `describe_types` and `get_index`. Locally the person can allow them with `trumly-mcp --write --allow-community`; the hosted server never does.

Every write returns `model` (`ok`, `errors`, `failed_rules`) and, for `put_node` and `rename_node`, `findings` for the node written, so the agent can fix what it broke straight away.

### Defaults and advice

The checker never fails on an undefined type, plane or link type, so a write tells the agent when it guessed. `put_node` and `rename_node` results may carry:

- `defaulted`: what the server filled in. A new node of a type that belongs on one plane (named by the type's `fields.plane`, or the only plane its nodes sit on) and names none gets that plane. A replacement is never given a plane.
- `advice`: a list of notes, each starting with a kind: `unknown_type`, `unknown_plane`, `unknown_link_type` (each with "did you mean"), `no_plane`, `unusual_plane`, `id_convention`. Advice never blocks a write.

### What the project says about itself

A project can say what it is for. With the `about` pack (part of the core bundle), one node with the id `overview` and type `overview` says what the project is for: `fields.purpose` (what is being built, for whom), `audience`, `owner`, `stage` (`define`, `design`, `plan` or `build`) and `source-of-truth`, and its body holds anything else. A rule checks the four required ones are filled in. How the project is written goes in nodes of type `convention` (a title and a Markdown body, such as "Short titles" or "Project decisions only"); a convention can point at the plane, type or node it governs with a `describes` link. `get_index` opens with the overview, then lists the conventions. A project with neither is told how to start, if the server can write.

### Revisions and conflicts

`get_node` returns a `rev` for the node's file. To replace or delete a node you pass that `rev` back as `base_rev`. If the node changed since (a person edited it, or another agent did) the call fails with a message starting `conflict`, and the agent reads the node again and redoes its change on top. A new node is created without `base_rev`; passing one for a node that no longer exists is also a conflict. Revisions only mean "the text is the same"; they carry no ordering.

### New ids

New ids use lower-case letters, digits, `-` and `_`, start and end with a letter or digit, are at most 100 characters, and have no `--` unless the node belongs to a pack. This keeps the file name working on every system. Existing ids are left as they are.

## Not yet

- A history of changes with undo and redo (see the backlog).
- The hosted server, its tokens and the edit log that says which changes an agent made.
