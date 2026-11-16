#![warn(missing_docs)]
//! The trumly engine: parse, validate and run rules over a typed graph of Markdown files.
//!
//! A project is a folder of Markdown files. Each file is a node: YAML front matter (an `id`, a `type`,
//! typed links and other fields) followed by a free-form body. This crate loads those files into a
//! [`model::Graph`], checks its structure, and evaluates the rules the project carries. It is pure:
//! it never reads a file or touches the network, so the caller supplies `(path, text)` pairs. That
//! is what lets the same code run in the command line tool, the MCP server, and the browser (as WASM).
//!
//! ```
//! use trumly_core::report::check;
//!
//! let files = vec![
//!     ("a.md".to_string(), "---\nid: a\ntype: note\n---\nHello.\n".to_string()),
//!     ("b.md".to_string(), "---\nid: b\ntype: note\nparent: missing\n---\n".to_string()),
//! ];
//! let report = check(files);
//! assert_eq!(report.graph.len(), 2);
//! // `b` names a parent that does not exist, so the structure check reports it.
//! assert!(report.structure.errors.iter().any(|f| f.kind == "missing_parent"));
//! assert_ne!(report.exit_code(), 0);
//! ```
//!
//! Where to start: [`report::check`] does the whole job; [`load`], [`structure`] and [`rules`] are
//! its stages; [`ops`] holds edits such as renaming a node and rewriting every link to it.

pub mod format;
pub mod load;
pub mod model;
pub mod ops;
pub mod parse;
pub mod report;
pub mod rules;
pub mod structure;
pub mod tools;
pub mod vocab;
