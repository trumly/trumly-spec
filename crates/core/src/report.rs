//! The result of checking a project, and how to show it. This is the one place that knows the shape
//! of `check --json`, so the CLI, the WASM build and the tests all agree.

use serde_json::{Value, json};

use crate::derive::{DerivedMap, derive_all, row_to_json};
use crate::load::{LoadError, load};
use crate::model::{Graph, Node};
use crate::rules::{Evaluation, Status, evaluate_with, unmatched_rules};
use crate::structure::{self, Finding, Structure};

/// The result of [`check`]: the loaded graph and everything found in it.
pub struct Report {
    /// The loaded project.
    pub graph: Graph,
    /// Files that could not be loaded.
    pub load_errors: Vec<LoadError>,
    /// Structure errors and warnings.
    pub structure: Structure,
    /// Rule results and ghosts.
    pub evaluation: Evaluation,
    /// Derived values by node id, for the nodes that have any.
    pub derived: DerivedMap,
}

/// Load, validate and run the rules over `(path, text)` pairs.
pub fn check(files: Vec<(String, String)>) -> Report {
    let (graph, load_errors) = load(files);
    let mut structure = structure::check(&graph);
    for id in unmatched_rules(&graph) {
        structure.warnings.push(Finding { node: id, kind: "rule_matches_nothing", reference: None });
    }
    structure.warnings.sort_by_key(|f| (f.node.clone(), f.kind, f.reference.clone().unwrap_or_default()));
    let derived = derive_all(&graph);
    let evaluation = evaluate_with(&graph, &derived);
    Report { graph, load_errors, structure, evaluation, derived }
}

/// A node as the JSON object used in `check --json`.
pub fn node_to_json(n: &Node) -> Value {
    let links: Vec<Value> = n
        .links
        .iter()
        .map(|l| {
            let mut m = json!({"type": l.kind, "to": l.to});
            if let Some(k) = &l.key {
                m["key"] = json!(k);
            }
            if !l.fields.is_empty() {
                m["fields"] = json!(l.fields);
            }
            m
        })
        .collect();
    json!({"id": n.id, "type": n.kind, "title": n.title, "parent": n.parent, "plane": n.plane,
           "fields": n.fields, "links": links, "body": n.body})
}

fn findings_to_json(list: &[Finding]) -> Value {
    Value::Array(list.iter().map(|f| json!({"node": f.node, "kind": f.kind, "ref": f.reference})).collect())
}

impl Report {
    /// The `check --json` document (section 6 of the spec). Object keys come out sorted.
    pub fn to_json(&self) -> Value {
        json!({
            "load_errors": self.load_errors.iter().map(|e| json!({"file": e.file, "kind": e.kind})).collect::<Vec<_>>(),
            "nodes": self.graph.nodes().map(|n| {
                let mut j = node_to_json(n);
                // only nodes that have derived values carry the key, so projects without any look as they always did
                if let Some(row) = self.derived.get(&n.id) {
                    j["derived"] = row_to_json(row);
                }
                j
            }).collect::<Vec<_>>(),
            "structure": {
                "errors": findings_to_json(&self.structure.errors),
                "warnings": findings_to_json(&self.structure.warnings),
            },
            "results": self.evaluation.results.iter().map(|r| json!({
                "node": r.node, "rule": r.rule, "severity": r.severity, "status": r.status.as_str()
            })).collect::<Vec<_>>(),
            "ghosts": self.evaluation.ghosts.iter().map(|g| json!({
                "node": g.node, "rule": g.rule, "link": g.link, "to_type": g.to_type, "plane": g.plane
            })).collect::<Vec<_>>(),
        })
    }

    /// 1 if anything is wrong enough to fail a build, else 0.
    pub fn exit_code(&self) -> i32 {
        let failed_error_rule =
            self.evaluation.results.iter().any(|r| r.status == Status::Fail && r.severity == "error");
        i32::from(!self.load_errors.is_empty() || !self.structure.errors.is_empty() || failed_error_rule)
    }

    /// The human-readable `check` output.
    pub fn human(&self) -> String {
        let mut out = String::new();
        let line = |out: &mut String, s: String| {
            out.push_str(&s);
            out.push('\n');
        };
        for e in &self.load_errors {
            line(&mut out, format!("ERROR  {}: {}", e.file, e.kind));
        }
        for f in &self.structure.errors {
            line(
                &mut out,
                format!("ERROR  {}: {} {}", f.node, f.kind, f.reference.as_deref().unwrap_or(""))
                    .trim_end()
                    .to_string(),
            );
        }
        for f in &self.structure.warnings {
            line(
                &mut out,
                format!("warn   {}: {} {}", f.node, f.kind, f.reference.as_deref().unwrap_or(""))
                    .trim_end()
                    .to_string(),
            );
        }
        for r in self.evaluation.results.iter().filter(|r| r.status == Status::Fail) {
            line(&mut out, format!("{:<6} {}: fails {}", r.severity, r.node, r.rule));
        }
        line(
            &mut out,
            format!(
                "{} nodes, {} load errors, {} errors, {} warnings",
                self.graph.len(),
                self.load_errors.len(),
                self.structure.errors.len(),
                self.structure.warnings.len()
            ),
        );
        out
    }

    /// Everything about one node: where it sits, what it links to, what links to it, how it fares.
    pub fn show(&self, id: &str) -> Option<String> {
        let n = self.graph.get(id)?;
        let mut out = format!("{}  ({})\n", n.title, n.kind.as_deref().unwrap_or("?"));
        out.push_str(&format!("id: {}\n", n.id));
        if let Some(p) = &n.parent {
            out.push_str(&format!("parent: {p}\n"));
        }
        if let Some(p) = &n.plane {
            out.push_str(&format!("plane: {p}\n"));
        }
        if !n.fields.is_empty() {
            out.push_str("fields:\n");
            for (k, v) in &n.fields {
                out.push_str(&format!("  {k}: {v}\n"));
            }
        }
        if !n.links.is_empty() {
            out.push_str("links out:\n");
            for l in &n.links {
                let key = l.key.as_deref().map(|k| format!("#{k}")).unwrap_or_default();
                out.push_str(&format!("  {} -> {}{}\n", l.kind, l.to, key));
            }
        }
        // Inbound links are derived: scan every node. Cheap at this size; an index can come later.
        let inbound: Vec<String> = self
            .graph
            .nodes()
            .flat_map(|o| o.links.iter().filter(|l| l.to == id).map(move |l| format!("  {} <- {}\n", l.kind, o.id)))
            .collect();
        if !inbound.is_empty() {
            out.push_str("links in:\n");
            inbound.iter().for_each(|s| out.push_str(s));
        }
        let results: Vec<_> = self.evaluation.results.iter().filter(|r| r.node == id).collect();
        if !results.is_empty() {
            out.push_str("rules:\n");
            for r in results {
                out.push_str(&format!("  {} {} ({})\n", r.status.as_str(), r.rule, r.severity));
            }
        }
        if !n.body.is_empty() {
            out.push_str(&format!("\n{}\n", n.body));
        }
        Some(out)
    }
}

fn many(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

impl Report {
    /// A short Markdown map of the whole project: counts, planes in order, the group tree, every node
    /// by plane and type, rules, and what is failing. It is meant to be read first by a person or an
    /// agent, then followed by opening only the nodes that matter. Generated, never edited.
    pub fn index(&self) -> String {
        use std::collections::BTreeMap;
        let nodes: Vec<&Node> = self.graph.nodes().collect();
        let kind = |n: &Node| n.kind.clone().unwrap_or_else(|| "(no type)".to_string());
        let fails =
            |id: &str| self.evaluation.results.iter().filter(|r| r.node == id && r.status == Status::Fail).count();
        let is = |n: &Node, k: &str| n.kind.as_deref() == Some(k);

        // Planes in declared order (`fields.order`), then by id, then gathered into stacks: a stack is a plane's
        // `fields.stack`, else the pack it came from, else `main`. Stacks keep the order of their first plane.
        let mut planes: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "plane")).collect();
        planes.sort_by(|a, b| {
            let o = |n: &Node| n.fields.get("order").and_then(Value::as_f64).unwrap_or(f64::MAX);
            o(a).partial_cmp(&o(b)).unwrap_or(std::cmp::Ordering::Equal).then(a.id.cmp(&b.id))
        });
        let stack_of = |p: &Node| -> String {
            let text = |k: &str| p.fields.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
            if let Some(s) = text("stack") {
                return s.to_string();
            }
            if p.fields.get("scratch").and_then(Value::as_bool) != Some(true) {
                if let Some(pk) = text("pack") {
                    return pk.split('@').next().unwrap_or("main").to_string();
                }
            }
            "main".to_string()
        };
        let mut stacks: Vec<String> = Vec::new();
        for p in &planes {
            let s = stack_of(p);
            if !stacks.contains(&s) {
                stacks.push(s);
            }
        }
        let mut grouped: Vec<&Node> = Vec::new();
        for s in &stacks {
            grouped.extend(planes.iter().copied().filter(|p| &stack_of(p) == s));
        }
        let planes = grouped;
        let groups: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "group")).collect();
        let rules: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "rule")).collect();
        let content: Vec<&Node> = nodes
            .iter()
            .copied()
            .filter(|n| !["plane", "group", "rule", "type", "link-type", "pack", "derived"].iter().any(|k| is(n, k)))
            .collect();
        let failing: usize = self.evaluation.results.iter().filter(|r| r.status == Status::Fail).count();

        let mut out = String::from("# Project index\n\n");
        out.push_str(&format!(
            "{} nodes: {} on {}, {}, {}. {} failing checks, {}, {}, {}.\n",
            self.graph.len(),
            content.len(),
            many(planes.len(), "plane"),
            many(groups.len(), "group"),
            many(rules.len(), "rule"),
            failing,
            many(self.structure.errors.len(), "structure error"),
            many(self.structure.warnings.len(), "warning"),
            many(self.load_errors.len(), "load error")
        ));
        out.push_str(
            "Generated by `trumly index`; do not edit. Open a node with `trumly show ID` or the MCP `get_node` tool.\n",
        );

        out.push_str("\n## Planes\n");
        let pack_nodes: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "pack")).collect();
        let mut number = 0;
        for s in &stacks {
            // the pack that gives the stack its name says what the stack is for
            let about = pack_nodes
                .iter()
                .find(|k| k.fields.get("name").and_then(Value::as_str) == Some(s.as_str()))
                .and_then(|k| k.fields.get("description"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let about = about.split_once(". ").map_or(about, |(first, _)| first).trim_end_matches('.'); // the first sentence is the summary
            out.push_str(&format!(
                "\n### Stack `{}`{}\n\n",
                s,
                if about.is_empty() { String::new() } else { format!(": {about}") }
            ));
            for p in planes.iter().filter(|p| &stack_of(p) == s) {
                number += 1;
                let n = content.iter().filter(|c| c.plane.as_deref() == Some(p.id.as_str())).count();
                let d = p.fields.get("description").and_then(Value::as_str).map(str::trim).unwrap_or("");
                out.push_str(&format!(
                    "{}. `{}` {}: {}{}\n",
                    number,
                    p.id,
                    p.title,
                    many(n, "node"),
                    if d.is_empty() { String::new() } else { format!(". {d}") }
                ));
            }
        }
        if planes.is_empty() {
            out.push_str("\nNone declared.\n");
        }

        if !groups.is_empty() {
            out.push_str("\n## Groups\n\n");
            let ids: std::collections::BTreeSet<&str> = groups.iter().map(|g| g.id.as_str()).collect();
            fn walk(
                out: &mut String,
                groups: &[&Node],
                nodes: &[&Node],
                ids: &std::collections::BTreeSet<&str>,
                parent: Option<&str>,
                depth: usize,
            ) {
                for g in groups.iter().filter(|g| match g.parent.as_deref() {
                    Some(p) if ids.contains(p) => Some(p) == parent,
                    _ => parent.is_none(),
                }) {
                    let members = nodes.iter().filter(|n| n.parent.as_deref() == Some(g.id.as_str())).count();
                    out.push_str(&format!(
                        "{}- `{}` {} ({})\n",
                        "  ".repeat(depth),
                        g.id,
                        g.title,
                        many(members, "member")
                    ));
                    if depth < 8 {
                        walk(out, groups, nodes, ids, Some(&g.id), depth + 1);
                    }
                }
            }
            walk(&mut out, &groups, &nodes, &ids, None, 0);
        }

        let packs: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "pack")).collect();
        if !packs.is_empty() {
            out.push_str("\n## Packs\n\n");
            for p in &packs {
                let f = |k: &str| p.fields.get(k).and_then(Value::as_str).unwrap_or("");
                out.push_str(&format!("- `{}` {}: {}\n", f("name"), f("version"), f("description")));
            }
        }

        out.push_str("\n## Nodes\n");
        let section = |out: &mut String, heading: &str, list: Vec<&Node>| {
            if list.is_empty() {
                return;
            }
            out.push_str(&format!("\n### {} ({})\n", heading, list.len()));
            let mut by_type: BTreeMap<String, Vec<&Node>> = BTreeMap::new();
            for n in list {
                by_type.entry(kind(n)).or_default().push(n);
            }
            for (t, ns) in by_type {
                out.push_str(&format!("\n**{}** ({})\n", t, ns.len()));
                for n in ns {
                    let mut line = format!("- `{}` {}", n.id, n.title);
                    if let Some(p) = &n.parent {
                        line.push_str(&format!(" [in {p}]"));
                    }
                    let f = fails(&n.id);
                    if f > 0 {
                        line.push_str(&format!(" ({f} failing)"));
                    }
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        };
        for p in &planes {
            section(
                &mut out,
                &format!("{}, plane `{}`", p.title, p.id),
                content.iter().copied().filter(|c| c.plane.as_deref() == Some(p.id.as_str())).collect(),
            );
        }
        let known: std::collections::BTreeSet<&str> = planes.iter().map(|p| p.id.as_str()).collect();
        section(
            &mut out,
            "Not on a plane",
            content.iter().copied().filter(|c| c.plane.as_deref().is_none_or(|p| !known.contains(p))).collect(),
        );

        if !rules.is_empty() {
            out.push_str("\n## Rules\n\n");
            for r in &rules {
                let on = r.fields.get("enabled").and_then(Value::as_bool) != Some(false);
                let sev = r.fields.get("severity").and_then(Value::as_str).unwrap_or("warn");
                let failed =
                    self.evaluation.results.iter().filter(|x| x.rule == r.id && x.status == Status::Fail).count();
                let state = if on { format!("{failed} failing") } else { "off".to_string() };
                out.push_str(&format!(
                    "- `{}` {} ({}, {})\n",
                    r.id,
                    r.fields.get("statement").and_then(Value::as_str).unwrap_or(&r.title),
                    sev,
                    state
                ));
            }
        }

        // calculated names (spec 7a): what each is called and the template it is built from, so an agent knows they exist
        let defs: Vec<&Node> = nodes.iter().copied().filter(|n| is(n, "derived")).collect();
        if !defs.is_empty() {
            out.push_str("\n## Derived values\n\n");
            for d in &defs {
                let on = d.fields.get("enabled").and_then(Value::as_bool) != Some(false);
                let field = d.fields.get("field-name").and_then(Value::as_str).filter(|s| !s.is_empty());
                out.push_str(&format!(
                    "- `{}` {}{}: `{}`{}\n",
                    d.id,
                    d.fields.get("name").and_then(Value::as_str).unwrap_or(&d.title),
                    field.map(|f| format!(" (value `{f}`)")).unwrap_or_default(),
                    d.fields.get("template").and_then(Value::as_str).unwrap_or(""),
                    if on { "" } else { " (off)" }
                ));
            }
        }

        let mut problems = String::new();
        for e in &self.load_errors {
            problems.push_str(&format!("- load error {}: {}\n", e.file, e.kind));
        }
        for f in &self.structure.errors {
            problems.push_str(
                &format!("- error `{}`: {} {}\n", f.node, f.kind, f.reference.as_deref().unwrap_or(""))
                    .replace(" \n", "\n"),
            );
        }
        for f in &self.structure.warnings {
            problems.push_str(
                &format!("- warning `{}`: {} {}\n", f.node, f.kind, f.reference.as_deref().unwrap_or(""))
                    .replace(" \n", "\n"),
            );
        }
        for r in self.evaluation.results.iter().filter(|r| r.status == Status::Fail) {
            problems.push_str(&format!("- {} `{}` fails `{}`\n", r.severity, r.node, r.rule));
        }
        if !problems.is_empty() {
            out.push_str("\n## Integrity checks\n\n");
            out.push_str(&problems);
        }
        out
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;

    fn file(path: &str, text: &str) -> (String, String) {
        (path.to_string(), text.to_string())
    }

    #[test]
    fn index_lists_planes_groups_nodes_and_findings() {
        let report = check(vec![
            file("build.md", "---\nid: build\ntype: plane\ntitle: Build\nfields: { order: 2 }\n---\n"),
            file("intent.md", "---\nid: intent\ntype: plane\ntitle: Intent\nfields: { order: 1 }\n---\n"),
            file("group-app.md", "---\nid: group-app\ntype: group\ntitle: App\n---\n"),
            file(
                "component-api.md",
                "---\nid: component-api\ntype: component\ntitle: API\nplane: build\nparent: group-app\n---\n",
            ),
            file(
                "rule-x.md",
                "---\nid: rule-x\ntype: rule\ntitle: x\nfields:\n  statement: Components need a note\n  severity: warn\n  applies_to: { type: component }\n  check: { require_fields: [note] }\n---\n",
            ),
        ]);
        let text = report.index();
        assert!(text.contains("1. `intent`"), "planes in declared order:\n{text}");
        assert!(text.contains("2. `build`"));
        assert!(text.contains("- `group-app` App (1 member)"));
        assert!(text.contains("- `component-api` API [in group-app] (1 failing)"));
        assert!(text.contains("- `rule-x` Components need a note (warn, 1 failing)"));
        assert!(text.contains("warn `component-api` fails `rule-x`"));
        assert!(text.find("1. `intent`") < text.find("2. `build`"));
    }

    #[test]
    fn index_gathers_planes_into_stacks_with_their_descriptions() {
        let report = check(vec![
            file(
                "biz--pack.md",
                "---\nid: biz--pack\ntype: pack\ntitle: Business\nfields:\n  name: business\n  version: 0.1.0\n  description: The commercial side.\n---\n",
            ),
            file(
                "a.md",
                "---\nid: a\ntype: plane\ntitle: Alpha\nfields: { order: 1, pack: 'system-design@0.3.0', description: Where things run. }\n---\n",
            ),
            file(
                "b.md",
                "---\nid: b\ntype: plane\ntitle: Beta\nfields: { order: 2, pack: 'business@0.1.0', description: Who we sell to. }\n---\n",
            ),
            file(
                "c.md",
                "---\nid: c\ntype: plane\ntitle: Gamma\nfields: { order: 3, pack: 'system-design@0.3.0' }\n---\n",
            ),
            file(
                "d.md",
                "---\nid: d\ntype: plane\ntitle: Delta\nfields: { order: 4, scratch: true, pack: 'x@1' }\n---\n",
            ),
        ]);
        let text = report.index();
        let at = |s: &str| text.find(s).unwrap_or_else(|| panic!("missing {s}:\n{text}"));
        assert!(at("### Stack `system-design`") < at("1. `a` Alpha: 0 nodes. Where things run."));
        assert!(at("1. `a`") < at("2. `c` Gamma: 0 nodes\n"), "planes of a stack sit together:\n{text}");
        assert!(at("2. `c`") < at("### Stack `business`: The commercial side"));
        assert!(at("### Stack `business`") < at("3. `b` Beta: 0 nodes. Who we sell to."));
        assert!(at("3. `b`") < at("### Stack `main`"), "a scratch plane goes in main:\n{text}");
        assert!(text.contains("4. `d` Delta"));
    }
}
