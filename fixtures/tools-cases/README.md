# Agent tool cases

Each file is one project and a list of tool calls with what they must return. The same cases run against the Rust core
(`crates/core/tests/tools.rs`), against the WASM build (`crates/wasm/js/verify-tools.mjs`) and against trumly cloud, so the
local, browser and hosted servers answer alike.

    {"tool": "search", "args": {...}, "expect": {...}}      expect is matched as a subset: every key given must be equal,
                                                            lists must be the same length with each item matched as a subset
    {"tool": "get_node", "args": {...}, "error": "text"}    the call fails and the message contains this text
    {"tool": "get_index", "args": {}, "text": ["a", "b"]}   the result is text containing each string

Only the pure tools are covered here (`get_index`, `get_node` without `path` and `rev`, `list_nodes`, `search`, `get_links`, `check`,
and the `_standing` and `_rename_edit` helpers). Revisions, paths, `status` and writing belong to each host and are tested there.
