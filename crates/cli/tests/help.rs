//! `trumly help` must name every command the binary answers to, so the help cannot fall behind the code.

use std::process::Command;

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_trumly")).args(args).output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn help_names_every_command_and_option() {
    let (help, _, code) = run(&["help"]);
    assert_eq!(code, 0);
    // only the top-level dispatch in `fn run`, not the sub-commands of pack or telemetry
    let main = include_str!("../src/main.rs");
    let main = &main[main.find("fn run(").unwrap()..];
    // every top-level command in the dispatch is in the help
    for line in main.lines().filter(|l| l.starts_with("        Some(\"")) {
        let name = line.trim().trim_start_matches("Some(\"").split('"').next().unwrap();
        if name.starts_with('-') || name == "help" {
            continue;
        }
        assert!(help.contains(&format!("  {name}")), "`trumly help` does not mention the command '{name}'");
    }
    for needle in [
        "telemetry",
        "rename type",
        "pack check",
        "pack status",
        "--use-existing",
        "--keep-ids",
        "--version",
        "DO_NOT_TRACK",
    ] {
        assert!(help.contains(needle), "help is missing {needle}");
    }
    assert_eq!(run(&["--help"]).0, help);
    assert_eq!(run(&["-h"]).0, help);
}

#[test]
fn an_unknown_command_or_none_prints_the_help_to_stderr_and_fails() {
    let (out, err, code) = run(&["frobnicate"]);
    assert_eq!((out.as_str(), code), ("", 2));
    assert!(err.contains("unknown command 'frobnicate'") && err.contains("usage: trumly COMMAND"));
    let (_, err, code) = run(&[]);
    assert_eq!(code, 2);
    assert!(err.contains("usage: trumly COMMAND"));
}

#[test]
fn version_prints_the_crate_version() {
    assert_eq!(run(&["--version"]).0.trim(), format!("trumly {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn quickstart_prints_the_guide() {
    let (out, _, code) = run(&["quickstart"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("# trumly quickstart"));
    assert!(out.contains("trumly pack add") && out.contains("trumly init"));
}
