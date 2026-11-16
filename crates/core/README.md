# trumly-core

The engine behind [trumly](https://spec.trumly.co): parse, validate and run rules over a typed graph of Markdown files.

A project is a folder of Markdown files. Each file is a node: YAML front matter (an `id`, a `type`, typed links and other fields) and a free-form body. This crate loads the files into a graph, checks its structure, and evaluates the rules the project carries.

It is **pure**: it never reads a file or touches the network. You pass `(path, text)` pairs, so the same code runs in a command line tool, a server, a test, or the browser as WebAssembly. It has two dependencies: `saphyr` (YAML) and `serde_json`.

```rust
use trumly_core::report::check;

let files = vec![
    ("a.md".to_string(), "---\nid: a\ntype: note\n---\nHello.\n".to_string()),
    ("b.md".to_string(), "---\nid: b\ntype: note\nparent: missing\n---\n".to_string()),
];
let report = check(files);
assert_eq!(report.graph.len(), 2);
assert!(report.structure.errors.iter().any(|f| f.kind == "missing_parent"));
```

- `report::check` does the whole job; `load`, `structure` and `rules` are its stages.
- `ops` holds edits that mean the same thing in every client, such as renaming a node and rewriting every link to it.
- `format` writes a node back as canonical Markdown, byte for byte the same everywhere.

## Conformance

This is the reference implementation of the [trumly specification](https://spec.trumly.co/format), not the only possible one. The conformance suite in the repository (`fixtures/cases`, inputs plus `expected.json`) says what any implementation must output, so a parser in another language can be checked against the same cases.

## Status

Version 0, a draft. The format and the API can change before 1.0.

Documentation: [spec.trumly.co/reference](https://spec.trumly.co/reference). Licence: Apache-2.0.
