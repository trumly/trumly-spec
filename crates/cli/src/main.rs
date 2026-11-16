//! The `trumly` command line. Run `trumly help` for every command and option (the text lives in `HELP` below).
//!
//! This is the adapter that touches the disk. All the logic lives in trumly-core.

use std::process::ExitCode;

mod render;

use trumly_core::report::{Report, check};
use trumly_fs::read_model;

fn load_report(dir: &str) -> Result<Report, String> {
    Ok(check(read_model(dir)?))
}

/// An explicit packs folder (`--packs DIR`); otherwise the library searches its usual places (see `trumly_fs::library`).
fn packs_dir(args: &[String]) -> Option<std::path::PathBuf> {
    args.iter().position(|a| a == "--packs").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from)
}

/// The positional arguments: everything that is not a flag or a flag's value.
fn positional(args: &[String]) -> Vec<&String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if a == "--packs" || a == "--only" {
            skip = true;
        } else if !a.starts_with("--") {
            out.push(a);
        }
    }
    out
}

fn telemetry(args: &[String]) -> Result<i32, String> {
    use trumly_fs::telemetry as t;
    match args.first().map(String::as_str).unwrap_or("status") {
        "status" => {
            println!(
                "usage data: {}{}",
                t::state().as_str(),
                if t::forced_off() {
                    "  (forced off by DO_NOT_TRACK or TRUMLY_TELEMETRY=off)"
                } else if t::chosen() == t::State::Unset {
                    "  (you have not been asked yet)"
                } else {
                    ""
                }
            );
            println!(
                "endpoint:   {}",
                t::endpoint().unwrap_or_else(|| "none (events stay on this computer)".to_string())
            );
            println!("queued:     {}", t::queued_count());
            Ok(0)
        }
        "on" => {
            t::set_state(t::State::On)?;
            println!("usage data is on. See exactly what is kept with: trumly telemetry show");
            Ok(0)
        }
        "off" => {
            t::set_state(t::State::Off)?;
            println!("usage data is off; the install id and anything unsent were deleted");
            Ok(0)
        }
        "show" => {
            println!(
                "Kept: a random install id, the version, your OS, what was done (like pack_added) and rounded counts.\nNever kept: node ids, titles, field values, file paths or any project content.\n"
            );
            let q = t::queued(1000);
            if q.is_empty() {
                println!("Nothing is queued.");
            }
            for e in q {
                println!("{e}");
            }
            Ok(0)
        }
        "send" => {
            let Some(url) = t::endpoint() else {
                return Err("events are kept on this computer: TRUMLY_TELEMETRY_URL is off".to_string());
            };
            if t::state() != t::State::On {
                return Err("usage data is not on".to_string());
            }
            let events = t::queued(200);
            if events.is_empty() {
                println!("nothing to send");
                return Ok(0);
            }
            let n = events.len();
            match ureq::post(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send_json(serde_json::json!({"events": events}))
            {
                Ok(_) => {
                    t::ack(n);
                    println!("sent {n} events");
                    Ok(0)
                }
                Err(e) => {
                    println!("could not reach {url} ({e}); the events are kept and will be sent later");
                    Ok(0)
                }
            }
        }
        _ => Err("usage: trumly telemetry [status|on|off|show|send]".to_string()),
    }
}

/// A readable default for the project's name: the folder's name with dashes and underscores as spaces, each word capitalised.
fn folder_name(dir: &std::path::Path) -> Option<String> {
    let abs = dir.canonicalize().ok()?;
    let base = abs.file_name()?.to_str()?;
    let words: Vec<String> = base
        .split(|c: char| matches!(c, '-' | '_') || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if words.is_empty() { None } else { Some(words.join(" ")) }
}

/// Break `text` into lines of at most `width` characters, at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for w in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + w.len() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// `trumly init`: the guided start's questions, asked at the terminal. Flags answer a question ahead of time; with no
/// terminal attached, every required answer has to come from flags.
fn init(args: &[String]) -> Result<i32, String> {
    use std::io::{BufRead, IsTerminal, Write};
    use trumly_fs::{library, start};
    let flag = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let mut skip = false;
    let mut dirs = Vec::new();
    for a in args {
        if skip {
            skip = false
        } else if a.starts_with("--") {
            skip = true
        } else {
            dirs.push(a)
        }
    }
    let dir = std::path::PathBuf::from(dirs.first().map_or(".", |s| s.as_str()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let lib = library::load(packs_dir(args).as_deref())?;
    let all = lib.packs();
    let extras: Vec<(String, String)> = all
        .iter()
        .filter(|p| p.role != "core" && lib.source_of(&p.name) != Some("community"))
        .map(|p| (p.name.clone(), p.use_when.clone()))
        .collect();
    let q = start::questions(&extras);
    let folder_default = folder_name(&dir);
    let tty = std::io::stdin().is_terminal();
    let stdin = std::io::stdin();
    let ask = |text: &str| -> Result<String, String> {
        print!("{text} ");
        std::io::stdout().flush().ok();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).map_err(|e| e.to_string())?;
        Ok(line.trim().to_string())
    };
    let mut given = serde_json::Map::new();
    for k in ["name", "purpose", "audience", "owner", "approach"] {
        if let Some(v) = flag(&format!("--{k}")) {
            given.insert(k.into(), serde_json::Value::String(v));
        }
    }
    if let Some(w) = flag("--with") {
        given.insert("packs".into(), serde_json::json!(w.split(',').filter(|s| !s.is_empty()).collect::<Vec<_>>()));
    }
    if tty {
        println!("Let's set up your project. A few short questions; the answers become its overview.\n");
        for item in q["questions"].as_array().into_iter().flatten() {
            let id = item["id"].as_str().unwrap_or("");
            if id == "kind" || given.contains_key(id) {
                continue;
            }
            let multi = item["answer"] == "multi";
            let options = item["options"].as_array().cloned().unwrap_or_default();
            if multi && options.is_empty() {
                continue;
            }
            println!("{}", item["ask"].as_str().unwrap_or(""));
            for (i, o) in options.iter().enumerate() {
                let hint = o["hint"].as_str().filter(|h| !h.is_empty()).map(|h| format!("  ({h})")).unwrap_or_default();
                println!("  {}) {}{hint}", i + 1, o["label"].as_str().unwrap_or(""));
            }
            let required = item["required"] == true;
            let fallback = if id == "name" { folder_default.clone() } else { None };
            loop {
                let prompt = if multi {
                    "numbers separated by commas, or enter for just the core >".to_string()
                } else if !options.is_empty() && item["answer"] == "text" {
                    "a number, or type your own >".to_string()
                } else if !options.is_empty() {
                    "a number >".to_string()
                } else if let Some(d) = &fallback {
                    format!("enter for {d} >")
                } else {
                    ">".to_string()
                };
                let mut line = ask(&prompt)?;
                if line.is_empty() {
                    if let Some(d) = &fallback {
                        line = d.clone();
                    }
                }
                let pick = |t: &str| {
                    t.trim()
                        .parse::<usize>()
                        .ok()
                        .and_then(|n| options.get(n.wrapping_sub(1)))
                        .and_then(|o| o["value"].as_str())
                        .map(str::to_string)
                };
                if multi {
                    let vals: Vec<String> = line.split(',').filter(|t| !t.trim().is_empty()).filter_map(pick).collect();
                    given.insert(id.into(), serde_json::json!(vals));
                    break;
                }
                let value = pick(&line)
                    .or_else(|| if item["answer"] == "text" && !line.is_empty() { Some(line.clone()) } else { None });
                match value {
                    Some(v) => {
                        given.insert(id.into(), serde_json::Value::String(v));
                        break;
                    }
                    None if !required => break,
                    None => println!("  that one is needed."),
                }
            }
            println!();
        }
        use trumly_fs::telemetry as tel;
        if tel::chosen() == tel::State::Unset && !tel::forced_off() {
            println!("Usage data is off. Turning it on sends only anonymous counts (which commands run,");
            println!("rounded sizes), never your project's content. It helps us prove that people are using");
            println!("trumly. See exactly what is sent with: trumly telemetry show");
            let yes = ask("Turn it on? [y/N]")?.to_ascii_lowercase();
            if yes == "y" || yes == "yes" {
                tel::set_state(tel::State::On)?;
                println!("Thank you. Turn it off any time: trumly telemetry off\n");
            } else {
                tel::set_state(tel::State::Off)?;
                println!("Left off. You can change your mind any time: trumly telemetry on\n");
            }
        }
    }
    if !given.contains_key("name") {
        if let Some(d) = &folder_default {
            given.insert("name".into(), serde_json::Value::String(d.clone()));
        }
    }
    let (a, missing) = start::read_answers(&serde_json::Value::Object(given));
    if !missing.is_empty() {
        return Err(format!(
            "still needed: {}. Give them as flags (--name, --purpose, --audience, --owner, --approach greenfield|brownfield), or run trumly init in a terminal.",
            missing.join(", ")
        ));
    }
    let done = start::start(&dir, &a, &all, &|n| extras.iter().any(|(e, _)| e == n))?;
    println!(
        "Set up {} in {}: {} nodes added from {}, plus the overview.",
        a.name,
        dir.display(),
        done.added,
        done.packs.join(", ")
    );
    println!("Next: trumly check {}   (or connect an AI agent and ask it to continue)", dir.display());
    Ok(0)
}

fn pack(args: &[String]) -> Result<i32, String> {
    use trumly_fs::library;
    use trumly_fs::pack::{add, detach, installed, read_pack, remove, status};
    let packs = packs_dir(args);
    let pos = positional(args);
    match pos.first().map(|s| s.as_str()) {
        Some("list") => {
            let model = pos.get(1).map_or(".", |s| s.as_str());
            let have = installed(std::path::Path::new(model)).unwrap_or_default();
            let lib = library::load(packs.as_deref())?;
            let name_w = lib.entries.iter().map(|e| e.pack.name.len()).max().unwrap_or(4).max(4);
            println!("{:<name_w$}  {:<8}  {:>5}  {:<10}  IN THIS FOLDER", "PACK", "VERSION", "NODES", "SOURCE");
            for e in &lib.entries {
                let p = &e.pack;
                let state = match have.iter().find(|(n, _)| *n == p.name) {
                    Some((_, v)) if *v == p.version => "added".to_string(),
                    Some((_, v)) => format!("has {v}"),
                    None => "-".to_string(),
                };
                let nodes = match p.files.len().saturating_sub(1) {
                    0 => "-".to_string(),
                    n => n.to_string(),
                };
                println!("{:<name_w$}  {:<8}  {:>5}  {:<10}  {}", p.name, p.version, nodes, e.source, state);
                for line in wrap(&p.description, 72) {
                    println!("    {line}");
                }
                if !p.includes.is_empty() {
                    println!("    includes {}", p.includes.join(", "));
                }
                if !p.requires.is_empty() {
                    println!("    needs {}", p.requires.join(", "));
                }
                println!();
            }
            println!("Add one with: trumly pack add NAME");
            Ok(0)
        }
        Some("add") => {
            let name = pos.get(1).ok_or("usage: trumly pack add NAME [DIR] [--only a,b] [--packs PACKS] [--force] [--use-existing | --replace-existing]")?;
            let model = std::path::Path::new(pos.get(2).map_or(".", |s| s.as_str()));
            let force = args.iter().any(|a| a == "--force");
            let only: Option<Vec<String>> = args.iter().position(|a| a == "--only").and_then(|i| args.get(i + 1)).map(|v| v.split(',').map(str::to_string).collect());
            // a path to a pack folder, or the name of one in the packs folder (with whatever it includes and needs)
            let results = if read_pack(std::path::Path::new(name.as_str())).is_ok() {
                let pack = read_pack(std::path::Path::new(name.as_str()))?;
                vec![(pack.name.clone(), add(&pack, model, force)?)]
            } else {
                let all = library::load(packs.as_deref())?.packs();
                let clash = if args.iter().any(|a| a == "--use-existing") { trumly_fs::pack::Clash::Skip } else if args.iter().any(|a| a == "--replace-existing") { trumly_fs::pack::Clash::Replace } else { trumly_fs::pack::Clash::Refuse };
                trumly_fs::pack::add_tree_with(&all, name, only.as_deref(), model, force, clash)?
            };
            for (pack, r) in results {
                println!("{pack}: {} added, {} updated", r.added.len(), r.updated.len());
                if !r.kept.is_empty() {
                    println!("  you changed these and the pack did too; yours were kept (--force takes the pack's): {}", r.kept.join(", "));
                }
                if !r.skipped.is_empty() {
                    println!("  left out, the project already defines them: {}", r.skipped.join(", "));
                }
                if !r.replaced.is_empty() {
                    println!("  replaced definitions that were already there: {}", r.replaced.join(", "));
                }
                if !r.removed.is_empty() {
                    println!("  dropped by the pack and removed: {}", r.removed.join(", "));
                }
                if !r.no_longer_in_pack.is_empty() {
                    println!("  no longer in the pack, but changed by you (left in place): {}", r.no_longer_in_pack.join(", "));
                }
            }
            Ok(0)
        }
        Some("remove") => {
            let name = pos.get(1).ok_or("usage: trumly pack remove NAME [DIR] [--force]")?;
            let model = std::path::Path::new(pos.get(2).map_or(".", |s| s.as_str()));
            let r = remove(name, model, args.iter().any(|a| a == "--force"))?;
            println!("{name}: removed {} nodes", r.removed.len());
            Ok(0)
        }
        Some("detach") => {
            let name = pos.get(1).ok_or("usage: trumly pack detach NAME [DIR] [--force]")?;
            let model = std::path::Path::new(pos.get(2).map_or(".", |s| s.as_str()));
            let r = detach(name, model, args.iter().any(|a| a == "--force"))?;
            println!("{name}: detached; {} nodes are now ordinary nodes of the project", r.kept.len());
            Ok(0)
        }
        Some("check") => {
            // `trumly pack check [NAME | FOLDER]`: the library, or one pack in it, or a pack folder you are working on
            let mut all = library::load(packs.as_deref())?.packs();
            let target = pos.get(1).map(|s| s.as_str());
            let name = match target {
                Some(t) if std::path::Path::new(t).join("").is_dir() && !all.iter().any(|p| p.name == t) => {
                    let p = read_pack(std::path::Path::new(t))?;
                    let n = p.name.clone();
                    all.retain(|x| x.name != n);
                    all.push(p);
                    Some(n)
                }
                other => other.map(str::to_string),
            };
            let mut found = trumly_fs::pack::lint(&all);
            found.extend(trumly_fs::pack::check_samples(&all));
            let problems: Vec<_> = found.into_iter().filter(|p| name.as_ref().is_none_or(|n| *n == p.pack)).collect();
            for p in &problems {
                println!("{:<8} {:<20} {}", p.severity, p.pack, p.message);
            }
            let errors = problems.iter().filter(|p| p.severity == "error").count();
            println!("{} packs checked, {} errors, {} warnings", name.as_ref().map_or(all.len(), |_| 1), errors, problems.len() - errors);
            Ok(if errors > 0 { 1 } else { 0 })
        }
        Some("status") => {
            let model = std::path::Path::new(pos.get(1).map_or(".", |s| s.as_str()));
            let all = status(model)?;
            if all.is_empty() {
                println!("no packs in this project");
            }
            for s in all {
                println!("{:<20} {}  {} as provided{}{}", s.name, s.version, s.clean,
                    if s.changed.is_empty() { String::new() } else { format!(", changed by you: {}", s.changed.join(", ")) },
                    if s.missing.is_empty() { String::new() } else { format!(", deleted by you: {}", s.missing.join(", ")) });
            }
            Ok(0)
        }
        _ => Err("usage:\n  trumly pack list [DIR] [--packs PACKS]\n  trumly pack add NAME [DIR] [--only a,b] [--packs PACKS] [--force] [--use-existing | --replace-existing]\n  trumly pack remove NAME [DIR] [--force]\n  trumly pack detach NAME [DIR] [--force]\n  trumly pack status [DIR]\n  trumly pack check [NAME | PACK-FOLDER] [--packs PACKS]".to_string()),
    }
}

/// `trumly rename node|type|link-type ...`: the edit is worked out by trumly-core, so the editor and an agent get the same result.
fn rename(args: &[String]) -> Result<i32, String> {
    let keep_ids = args.iter().any(|a| a == "--keep-ids");
    let pos = positional(args);
    let usage = "usage:\n  trumly rename node FROM TO [DIR]\n  trumly rename type OLD NEW [DIR] [--keep-ids]\n  trumly rename link-type OLD NEW [DIR]";
    let (what, a, b) = match (pos.first(), pos.get(1), pos.get(2)) {
        (Some(w), Some(a), Some(b)) => (w.as_str(), a.as_str(), b.as_str()),
        _ => return Err(usage.into()),
    };
    let dir = pos.get(3).map_or(".", |d| d.as_str());
    let (graph, _) = trumly_core::load::load(read_model(dir)?);
    let edit = match what {
        "node" => trumly_core::ops::rename_node(&graph, a, b)?,
        "type" | "link-type" => {
            let def = graph
                .nodes()
                .find(|n| n.kind.as_deref() == Some(what) && n.fields.get("name").and_then(|v| v.as_str()) == Some(a))
                .ok_or(format!("the project defines no {what} called '{a}'"))?;
            trumly_core::ops::rename_def(&graph, &def.id.clone(), b, what == "type" && !keep_ids)?
        }
        _ => return Err(usage.into()),
    };
    let (w, r) = trumly_fs::apply_edit(dir, &edit)?;
    println!("renamed {a} to {b}: {w} files written, {r} removed");
    Ok(0)
}

/// Everything the command line can do. Kept in one place so `trumly help` and the usage error say the same thing.
fn help() -> String {
    HELP.to_string()
}

const HELP: &str = "trumly  a system's design, as a typed graph in git

usage: trumly COMMAND [ARGS]      (new here? run: trumly quickstart)

Look at a project (DIR defaults to the current folder)
  check [--json] [--no-color] [DIR]
                              run every rule and list the integrity check results (exit 1 when one fails)
  show ID [DIR]               one node: its links in and out, and its rule results
  index [DIR] [--write]       a Markdown map of the whole project; --write saves .trumly/INDEX.md

Change a project
  init [DIR] [--name N --purpose P --audience A --owner O --approach greenfield|brownfield --with a,b] [--packs PACKS]
                              set a new project up: asks the same short questions as the cloud's guided start
                              (anything given as a flag is not asked), adds the core pack and writes the overview
  rename node FROM TO [DIR]              give a node a new id; links, parents and rule filters follow
  rename type OLD NEW [DIR] [--keep-ids] rename a type everywhere it is used
  rename link-type OLD NEW [DIR]         rename a link type everywhere it is used

Packs (ready-made definitions: types, link types, rules, planes)
  pack list [DIR] [--packs PACKS]                    the packs available, and which the project has
  pack add NAME [DIR] [--only a,b] [--packs PACKS] [--force] [--use-existing | --replace-existing]
                                                     add a pack, with what it needs and includes
  pack remove NAME [DIR] [--force]                   take a pack out of the project
  pack detach NAME [DIR] [--force]                   let go of a pack but keep its nodes as your own
  pack status [DIR]                                  which packs the project has and what you changed in them
  pack check [NAME | PACK-FOLDER] [--packs PACKS]    lint packs: clashing names, stray nodes, missing types; every pack needs a passing example

Anonymous usage data (off until you turn it on)
  telemetry status            whether it is on, and where it goes
  telemetry on | off          turn it on or off; off also deletes the install id and anything unsent
  telemetry show              exactly what has been kept and would be sent
  telemetry send              send what is waiting now, to telemetry.trumly.co unless TRUMLY_TELEMETRY_URL says
                              otherwise (TRUMLY_TELEMETRY_URL=off keeps events on this computer)
  DO_NOT_TRACK=1 or TRUMLY_TELEMETRY=off always wins.

Colour is off for NO_COLOR, --no-color and output that is not a terminal; the words say the same either way.

Other
  quickstart                  a short guide to getting going (also in docs/QUICKSTART.md)
  help, --help, -h            this text
  --version, -V               the version

--packs PACKS is a folder of packs (or set TRUMLY_PACKS). Without it trumly looks in ./packs, then
~/.trumly/packs, ~/.trumly/community, and the core packs built in (the rest come from a folder or the registry).
";

/// Runs the command and returns the process exit code. `?` hands any `Err(String)` straight up.
fn run(args: &[String]) -> Result<i32, String> {
    match args.first().map(String::as_str) {
        Some("help" | "--help" | "-h") => {
            print!("{}", help());
            Ok(0)
        }
        Some("quickstart") => {
            print!("{}", include_str!("../../../docs/QUICKSTART.md"));
            Ok(0)
        }
        Some("--version" | "-V") => {
            println!("trumly {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Some("check") => {
            let json = args.iter().any(|a| a == "--json");
            let dir = args[1..].iter().find(|a| !a.starts_with("--")).map_or(".", String::as_str);
            let report = load_report(dir)?;
            if json {
                let text = serde_json::to_string_pretty(&report.to_json()).map_err(|e| e.to_string())?;
                println!("{text}");
            } else {
                let name = std::fs::canonicalize(dir)
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| dir.to_string());
                print!("{}", render::check_text(&report, &name, render::Style::detect(args)));
            }
            let fails = |sev: bool| {
                report
                    .evaluation
                    .results
                    .iter()
                    .filter(|r| r.status == trumly_core::rules::Status::Fail && (r.severity == "error") == sev)
                    .count()
            };
            let (e, w) = (
                report.load_errors.len() + report.structure.errors.len() + fails(true),
                report.structure.warnings.len() + fails(false),
            );
            trumly_fs::telemetry::track(
                "check_run",
                "cli",
                serde_json::json!({"errors": trumly_fs::telemetry::bucket(e), "warnings": trumly_fs::telemetry::bucket(w)}),
            );
            Ok(report.exit_code())
        }
        Some("show") => {
            let id = args.get(1).ok_or("usage: trumly show ID [DIR]")?;
            let report = load_report(args.get(2).map_or(".", String::as_str))?;
            let text = report.show(id).ok_or(format!("no node with id '{id}'"))?;
            print!("{text}");
            Ok(0)
        }
        Some("index") => {
            let write = args.iter().any(|a| a == "--write");
            let dir = args[1..].iter().find(|a| !a.starts_with("--")).map_or(".", String::as_str);
            let text = load_report(dir)?.index();
            if write {
                let path = std::path::Path::new(dir).join(".trumly").join("INDEX.md");
                std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                std::fs::write(&path, &text).map_err(|e| e.to_string())?;
                println!("wrote {}", path.display());
            } else {
                print!("{text}");
            }
            Ok(0)
        }
        Some("rename") => rename(&args[1..]),
        Some("init") => init(&args[1..]),
        Some("pack") => pack(&args[1..]),
        Some("telemetry") => telemetry(&args[1..]),
        None => Err(help().trim_end().to_string()),
        Some(other) => Err(format!("unknown command '{other}'\n\n{}", help())),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}
