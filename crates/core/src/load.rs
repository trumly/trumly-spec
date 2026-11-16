//! Builds a `Graph` from a set of files. Still pure: the caller reads the files and hands us
//! (path, text) pairs, so this works the same from the CLI, a test, or WASM in a browser.

use crate::model::Graph;
use crate::parse::parse_node;

/// One file that could not be loaded, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    /// Path relative to the project root, with `/` separators.
    pub file: String, // path relative to the project root, with `/` separators
    /// The name used in `check --json`, for example `missing_id`.
    pub kind: &'static str,
}

/// The filename without the directory and without `.md`: "sub/a.md" -> "a".
fn stem(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".md").unwrap_or(name)
}

/// Notes that live next to a project: any `README.md`, and anything whose name or folder starts with a dot.
pub fn ignored(path: &str) -> bool {
    let mut parts = path.split('/');
    path.rsplit('/').next() == Some("README.md") || parts.any(|p| p.starts_with('.'))
}

/// Load every `.md` file. Files that fail are reported and skipped; the rest still load.
pub fn load(mut files: Vec<(String, String)>) -> (Graph, Vec<LoadError>) {
    // Sorted by path so "first wins" for duplicate ids is the same on every machine.
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files.retain(|(path, _)| path.ends_with(".md") && !ignored(path));

    let mut graph = Graph::new();
    let mut errors = Vec::new();

    for (path, text) in files {
        let kind = match parse_node(&text) {
            Err(e) => Some(e.as_str()),
            Ok(node) if node.id != stem(&path) => Some("id_filename_mismatch"),
            // `insert` hands the node back unused when the id is taken, so we ask it first.
            Ok(node) => graph.insert(node).err().map(|_| "duplicate_id"),
        };
        if let Some(kind) = kind {
            errors.push(LoadError { file: path, kind });
        }
    }
    errors.sort_by(|a, b| (&a.file, a.kind).cmp(&(&b.file, b.kind)));
    (graph, errors)
}
