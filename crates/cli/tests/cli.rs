//! Runs the real `trumly` binary against the fixtures: the end-to-end check.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use trumly_core::rules::json_eq;

fn cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cases")
}

fn trumly(args: &[&str]) -> std::process::Output {
    // CARGO_BIN_EXE_<name> is set by cargo to the path of the freshly built binary.
    Command::new(env!("CARGO_BIN_EXE_trumly")).args(args).output().unwrap()
}

#[test]
fn json_output_matches_expected_for_every_case() {
    let mut n = 0;
    for entry in std::fs::read_dir(cases()).unwrap() {
        let case = entry.unwrap().path();
        let nodes = case.join("nodes");
        if !nodes.is_dir() {
            continue;
        }
        let out = trumly(&["check", "--json", nodes.to_str().unwrap()]);
        let got: Value = serde_json::from_slice(&out.stdout).unwrap();
        let exp: Value = serde_json::from_str(&std::fs::read_to_string(case.join("expected.json")).unwrap()).unwrap();
        assert!(json_eq(&got, &exp), "{}", case.display());
        n += 1;
    }
    assert!(n >= 20);
}

#[test]
fn exit_codes() {
    let ok = cases().join("valid-minimal/nodes");
    let bad = cases().join("missing-parent/nodes");
    assert_eq!(trumly(&["check", ok.to_str().unwrap()]).status.code(), Some(0));
    assert_eq!(trumly(&["check", bad.to_str().unwrap()]).status.code(), Some(1));
    assert_eq!(trumly(&["nonsense"]).status.code(), Some(2));
}

#[test]
fn show_prints_links_in_and_out() {
    let dir = cases().join("ex-ops-api/nodes");
    let out = trumly(&["show", "decision-fastapi", dir.to_str().unwrap()]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("links out:") && text.contains("governs -> component-ops-api"), "{text}");
    let out = trumly(&["show", "component-ops-api", dir.to_str().unwrap()]);
    assert!(String::from_utf8(out.stdout).unwrap().contains("governs <- decision-fastapi"));
    assert_eq!(trumly(&["show", "nope", dir.to_str().unwrap()]).status.code(), Some(2));
}
