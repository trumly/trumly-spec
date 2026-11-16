//! How `trumly check` reads on a terminal: the trumly mark and context on one line, a fixed four-character status
//! column (pass, warn, FAIL, so it reads with colour off), and one summary line. Colour is decoration on top of
//! the words and is dropped for NO_COLOR, `--no-color`, a dumb terminal and anything that is not a terminal.

use std::collections::BTreeMap;
use std::io::IsTerminal;

use trumly_core::report::Report;
use trumly_core::rules::Status;

/// What the terminal can show.
#[derive(Clone, Copy)]
pub struct Style {
    pub color: bool,
    pub truecolor: bool,
    pub unicode: bool,
}

impl Style {
    pub fn detect(args: &[String]) -> Style {
        let dumb = std::env::var("TERM").is_ok_and(|t| t == "dumb");
        let off = args.iter().any(|a| a == "--no-color")
            || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
            || dumb
            || !std::io::stdout().is_terminal();
        let truecolor = std::env::var("COLORTERM").is_ok_and(|c| c == "truecolor" || c == "24bit");
        let utf8 = ["LC_ALL", "LC_CTYPE", "LANG"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty())
            .is_none_or(|v| v.to_lowercase().contains("utf"));
        Style { color: !off, truecolor, unicode: !dumb && utf8 }
    }
    #[cfg(test)]
    pub const PLAIN: Style = Style { color: false, truecolor: false, unicode: true };

    /// the brand's terminal colours: 24-bit values from the design system, or the nearest of the 16
    fn paint(&self, role: &str, text: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        let (rgb, basic) = match role {
            "mark" => ("104;208;217", "36"),
            "pass" => ("125;213;150", "32"),
            "warn" => ("240;176;90", "33"),
            "fail" => ("254;161;150", "91;1"),
            "dim" => ("158;174;182", "2"),
            "bold" => ("", "1"),
            _ => return text.to_string(),
        };
        let code = if self.truecolor && !rgb.is_empty() {
            format!("38;2;{rgb}{}", if role == "fail" { ";1" } else { "" })
        } else {
            basic.to_string()
        };
        format!("\x1b[{code}m{text}\x1b[0m")
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// the `check` report, in the brand's terminal layout
pub fn check_text(report: &Report, name: &str, style: Style) -> String {
    let mark = style.paint("mark", if style.unicode { "◇" } else { "<>" });
    let planes = report.graph.nodes().filter(|n| n.kind.as_deref() == Some("plane")).count();
    let packs = report
        .graph
        .nodes()
        .filter_map(|n| n.fields.get("pack").and_then(|v| v.as_str()))
        .map(|p| p.split('@').next().unwrap_or(p).to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let mut out = format!(
        "{mark} {}  {}\n\n",
        style.paint("bold", "trumly check"),
        style.paint(
            "dim",
            &format!(
                "{name} · {} · {} · {}",
                plural(report.graph.len(), "node", "nodes"),
                plural(planes, "plane", "planes"),
                plural(packs, "pack", "packs")
            )
        )
    );
    let row = |out: &mut String, word: &str, role: &str, text: &str, dim: &str| {
        let pad = style.paint(role, word);
        out.push_str(&format!(
            "  {pad}  {text}{}\n",
            if dim.is_empty() { String::new() } else { format!("  {}", style.paint("dim", dim)) }
        ));
    };
    let detail = |out: &mut String, id: &str, rest: &str| {
        out.push_str(&format!("        {}  {}\n", style.paint("dim", id), rest))
    };

    let (mut failing, mut warnings, mut notes, mut passing) = (0usize, 0usize, 0usize, 0usize);

    // rules: one row each, with the nodes it fails under it
    struct Row {
        title: String,
        id: String,
        pass: usize,
        fails: Vec<String>,
        severity: String,
    }
    let mut rules: BTreeMap<String, Row> = BTreeMap::new();
    for r in &report.evaluation.results {
        let e = rules.entry(r.rule.clone()).or_insert_with(|| Row {
            title: report.graph.get(&r.rule).map_or(r.rule.clone(), |n| n.title.clone()),
            id: r.rule.clone(),
            pass: 0,
            fails: vec![],
            severity: r.severity.clone(),
        });
        if r.status == Status::Fail {
            e.fails.push(r.node.clone());
        } else {
            e.pass += 1;
        }
    }
    let width = rules.values().map(|r| r.title.chars().count()).max().unwrap_or(0).min(48);
    let pad = |s: &str| format!("{s}{}", " ".repeat(width.saturating_sub(s.chars().count())));
    for r in rules.values().filter(|r| r.fails.is_empty()) {
        passing += r.pass;
        row(&mut out, "pass", "pass", &pad(&r.title), &format!("{}  {}", r.id, plural(r.pass, "node", "nodes")));
    }
    for r in rules.values().filter(|r| !r.fails.is_empty()) {
        passing += r.pass;
        let (word, role) = match r.severity.as_str() {
            "error" => ("FAIL", "fail"),
            "info" => ("info", "dim"),
            _ => ("warn", "warn"),
        };
        match r.severity.as_str() {
            "error" => failing += r.fails.len(),
            "info" => notes += r.fails.len(),
            _ => warnings += r.fails.len(),
        }
        row(&mut out, word, role, &pad(&r.title), &r.id);
        for n in &r.fails {
            detail(&mut out, n, &report.graph.get(n).map_or(String::new(), |x| x.title.clone()));
        }
    }
    for e in &report.load_errors {
        failing += 1;
        row(&mut out, "FAIL", "fail", &format!("{}  {}", e.file, e.kind), "");
    }
    for f in &report.structure.errors {
        failing += 1;
        row(
            &mut out,
            "FAIL",
            "fail",
            format!("{}  {} {}", f.node, f.kind, f.reference.as_deref().unwrap_or("")).trim_end(),
            "",
        );
    }
    for f in &report.structure.warnings {
        warnings += 1;
        row(
            &mut out,
            "warn",
            "warn",
            format!("{}  {} {}", f.node, f.kind, f.reference.as_deref().unwrap_or("")).trim_end(),
            "",
        );
    }
    if rules.is_empty()
        && report.load_errors.is_empty()
        && report.structure.errors.is_empty()
        && report.structure.warnings.is_empty()
    {
        out.push_str(&format!("  {}\n", style.paint("dim", "no rules to run")));
    }

    out.push('\n');
    let mut parts = vec![
        style.paint(if failing > 0 { "fail" } else { "dim" }, &format!("{failing} failing")),
        style.paint(if warnings > 0 { "warn" } else { "dim" }, &plural(warnings, "warning", "warnings")),
    ];
    if notes > 0 {
        parts.push(style.paint("dim", &format!("{notes} info")));
    }
    parts.push(style.paint(if passing > 0 { "pass" } else { "dim" }, &format!("{passing} passing")));
    out.push_str(&format!("{}   {}\n", parts.join(" · "), style.paint("dim", &format!("exit {}", report.exit_code()))));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use trumly_core::report::check;

    fn nodes(list: &[(&str, &str)]) -> Report {
        check(list.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect())
    }

    #[test]
    fn reads_the_same_with_colour_off() {
        let r = nodes(&[("a.md", "---\nid: a\ntype: thing\n---\n"), ("b.md", "---\nid: b\ntype: thing\n---\n")]);
        let text = check_text(&r, "demo", Style::PLAIN);
        assert!(text.starts_with("◇ trumly check  demo · 2 nodes · 0 planes · 0 packs\n"), "{text}");
        assert!(!text.contains('\x1b'));
        assert!(text.trim_end().lines().last().unwrap().contains("exit 0"), "{text}");
    }

    #[test]
    fn colour_adds_codes_but_not_words() {
        let r = nodes(&[("a.md", "---\nid: a\ntype: thing\n---\n")]);
        let plain = check_text(&r, "demo", Style::PLAIN);
        let colour = check_text(&r, "demo", Style { color: true, truecolor: false, unicode: true });
        assert!(colour.contains('\x1b'));
        let stripped: String = {
            let mut s = String::new();
            let mut it = colour.chars().peekable();
            while let Some(c) = it.next() {
                if c == '\x1b' {
                    for d in it.by_ref() {
                        if d == 'm' {
                            break;
                        }
                    }
                } else {
                    s.push(c);
                }
            }
            s
        };
        assert_eq!(stripped, plain);
    }

    #[test]
    fn ascii_mark_when_unicode_is_off() {
        let r = nodes(&[("a.md", "---\nid: a\ntype: thing\n---\n")]);
        assert!(
            check_text(&r, "demo", Style { color: false, truecolor: false, unicode: false })
                .starts_with("<> trumly check")
        );
    }
}
