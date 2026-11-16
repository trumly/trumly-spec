//! Runs every case under fixtures/tools-cases against `trumly_core::tools`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use trumly_core::report::check;
use trumly_core::tools;

fn cases_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tools-cases")
}

/// `want` is a subset of `got`: objects need every key given, lists the same length with each item a subset.
fn subset(got: &Value, want: &Value, at: &str) -> Result<(), String> {
    match (got, want) {
        (Value::Object(g), Value::Object(w)) => {
            w.iter().try_for_each(|(k, wv)| subset(g.get(k).unwrap_or(&Value::Null), wv, &format!("{at}.{k}")))
        }
        (Value::Array(g), Value::Array(w)) => {
            if g.len() != w.len() {
                return Err(format!("{at}: expected {} items, got {}: {got}", w.len(), g.len()));
            }
            g.iter().zip(w).enumerate().try_for_each(|(i, (gv, wv))| subset(gv, wv, &format!("{at}[{i}]")))
        }
        _ if got == want => Ok(()),
        _ => Err(format!("{at}: expected {want}, got {got}")),
    }
}

#[test]
fn every_tool_case_gives_the_expected_answer() {
    let mut ran = 0;
    let mut names: Vec<_> = fs::read_dir(cases_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    names.sort();
    for path in names {
        let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let files: Vec<(String, String)> = case["files"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
            .collect();
        let report = check(files);
        for (i, call) in case["calls"].as_array().unwrap().iter().enumerate() {
            let label = format!("{} call {i} ({})", path.file_name().unwrap().to_string_lossy(), call["tool"]);
            let got = tools::call(&report, call["tool"].as_str().unwrap(), &call["args"])
                .unwrap_or_else(|| panic!("{label}: no such tool"));
            if let Some(want) = call["error"].as_str() {
                let e = got.expect_err(&format!("{label}: expected an error"));
                assert!(e.contains(want), "{label}: error '{e}' does not contain '{want}'");
            } else {
                let v = got.unwrap_or_else(|e| panic!("{label}: {e}"));
                if let Some(texts) = call["text"].as_array() {
                    let t = v.as_str().unwrap_or_else(|| panic!("{label}: expected text"));
                    for s in texts {
                        assert!(t.contains(s.as_str().unwrap()), "{label}: text lacks '{s}'");
                    }
                } else {
                    subset(&v, &call["expect"], "result").unwrap_or_else(|e| panic!("{label}: {e}"));
                }
            }
            ran += 1;
        }
    }
    assert!(ran > 10, "only {ran} calls ran");
}
