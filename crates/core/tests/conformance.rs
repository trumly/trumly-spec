//! Runs every case under fixtures/cases and compares the whole `check --json` output
//! (minus formatting) with the reference's expected.json.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use trumly_core::report::check;
use trumly_core::rules::json_eq;
fn cases_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cases")
}

/// All .md files under `root`, as (path relative to root with `/`, text).
fn read_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            read_files(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.push((rel, fs::read_to_string(&path).unwrap()));
        }
    }
}

#[test]
fn output_matches_the_reference() {
    let mut names: Vec<_> = fs::read_dir(cases_dir()).unwrap().map(|e| e.unwrap().path()).collect();
    names.sort();
    let mut checked = 0;
    for case in names.iter().filter(|p| p.join("expected.json").exists()) {
        let nodes_dir = case.join("nodes");
        let mut files = Vec::new();
        read_files(&nodes_dir, &nodes_dir, &mut files);

        let report = check(files);
        let got = report.to_json();
        let graph = &report.graph;
        let expected: Value = serde_json::from_str(&fs::read_to_string(case.join("expected.json")).unwrap()).unwrap();
        let name = case.file_name().unwrap().to_string_lossy();
        // Compare section by section so a failure says which part differs.
        for part in ["load_errors", "nodes", "structure", "results", "ghosts"] {
            assert!(
                json_eq(&got[part], &expected[part]),
                "{part} in {name}\n got: {}\n exp: {}",
                got[part],
                expected[part]
            );
        }

        // Writing a node out and reading it back must give the same node (the formatter's contract).
        for n in graph.nodes() {
            let text = trumly_core::format::node_to_markdown(n);
            let back =
                trumly_core::parse::parse_node(&text).unwrap_or_else(|e| panic!("{name}/{}: {e:?}\n{text}", n.id));
            assert!(
                json_eq(&trumly_core::report::node_to_json(n), &trumly_core::report::node_to_json(&back)),
                "round trip of {name}/{}\n{text}",
                n.id
            );
        }
        checked += 1;
    }
    assert!(checked >= 20, "only found {checked} cases");
}
