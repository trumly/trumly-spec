//! Structural checks: is the graph well-formed, before any rules run?
//! Mirrors `structure` in reference/core.py and section 4 of spec/FORMAT.md.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::model::Graph;
use crate::names::{Kind, Names, def_ns};

/// The longest name (title) a node should have, in characters: it has to fit a label on the map, a cell in a table and a line in a listing.
pub const TITLE_MAX: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
/// One problem found in the structure of the graph.
pub struct Finding {
    /// Id of the node the finding is about.
    pub node: String,
    /// The finding's name as used in `check --json`, for example `missing_parent`.
    pub kind: &'static str,
    /// The related id, for example a parent that does not exist.
    pub reference: Option<String>, // the related id, e.g. the parent that does not exist
}

#[derive(Debug, Default)]
/// The findings of a structure check.
pub struct Structure {
    /// Problems that make the project invalid.
    pub errors: Vec<Finding>,
    /// Problems worth fixing that do not make the project invalid.
    pub warnings: Vec<Finding>,
}

/// Walk up the parent chain from `id`. True if we ever come back to a node we have already seen.
/// A chain that ends at a missing node simply stops; that is `missing_parent`, reported elsewhere.
fn reaches_cycle(graph: &Graph, id: &str) -> bool {
    let mut seen = BTreeSet::new();
    let mut cur = Some(id);
    while let Some(c) = cur {
        let Some(node) = graph.get(c) else { return false };
        if !seen.insert(c) {
            return true; // insert returns false when the value was already there
        }
        cur = node.parent.as_deref();
    }
    false
}

/// The namespace a pack's nodes use: its manifest's `namespace`, else its name.
pub(crate) fn pack_namespace(graph: &Graph, name: &str) -> String {
    graph
        .nodes()
        .filter(|n| n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(|v| v.as_str()) == Some(name))
        .map(|n| {
            n.fields.get("namespace").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or(name).to_string()
        })
        .next()
        .unwrap_or_else(|| name.to_string())
}

/// A `type` or `link-type` whose `fields.name` another node of the same kind and namespace also has: the name would mean two things.
fn duplicate_definition(graph: &Graph, n: &crate::model::Node) -> Option<String> {
    let kind = n.kind.as_deref().filter(|k| *k == "type" || *k == "link-type")?;
    let name = n.fields.get("name").and_then(|v| v.as_str())?;
    let twin = graph.nodes().any(|m| {
        m.id != n.id
            && m.kind.as_deref() == Some(kind)
            && m.fields.get("name").and_then(|v| v.as_str()) == Some(name)
            && def_ns(&m.id) == def_ns(&n.id)
    });
    twin.then(|| name.to_string())
}

/// The (kind, name) pairs a `rule`, `derived` or `link-type` node uses: type names and link type names.
fn names_in(n: &crate::model::Node) -> Vec<(Kind, String)> {
    fn strs(v: Option<&Value>) -> Vec<String> {
        match v {
            Some(Value::String(s)) => vec![s.clone()],
            Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
            _ => Vec::new(),
        }
    }
    fn check_names(check: &Value, out: &mut Vec<(Kind, String)>) {
        let Some(map) = check.as_object() else { return };
        for (key, val) in map {
            match key.as_str() {
                "acyclic" => out.extend(strs(Some(val)).into_iter().map(|s| (Kind::Link, s))),
                "any_of" => {
                    for it in val.as_array().into_iter().flatten() {
                        check_names(it, out);
                    }
                }
                "require_link" | "forbid_link" | "require_link_fields" | "path" => {
                    let list: Vec<&Value> = match val {
                        Value::Array(a) => a.iter().collect(),
                        v => vec![v],
                    };
                    for it in list.into_iter().filter_map(Value::as_object) {
                        out.extend(strs(it.get("type")).into_iter().map(|s| (Kind::Link, s)));
                        for k in ["node_type", "to_type", "ancestor_type"] {
                            out.extend(strs(it.get(k)).into_iter().map(|s| (Kind::Type, s)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let f = &n.fields;
    let mut out = Vec::new();
    let applies = f.get("applies_to").and_then(Value::as_object).and_then(|w| w.get("type"));
    match n.kind.as_deref() {
        Some("rule") => {
            out.extend(strs(applies).into_iter().map(|s| (Kind::Type, s)));
            if let Some(c) = f.get("check") {
                check_names(c, &mut out);
            }
        }
        Some("derived") => {
            out.extend(strs(applies).into_iter().map(|s| (Kind::Type, s)));
            if let Some(t) = f.get("template").and_then(Value::as_str) {
                out.extend(crate::derive::template_links(t).into_iter().map(|s| (Kind::Link, s)));
            }
        }
        Some("link-type") => {
            for key in ["from", "to"] {
                out.extend(strs(f.get(key)).into_iter().map(|s| (Kind::Type, s)));
            }
        }
        _ => {}
    }
    out
}

/// Fields the node's type definition marks `required: true` that it leaves unset (absent, null or ""). A required field named `body`
/// is the Markdown body itself: it is missing while the body is blank.
fn missing_required(graph: &Graph, names: &Names, n: &crate::model::Node) -> Vec<String> {
    const RESERVED: [&str; 7] = ["plane", "rule", "group", "type", "link-type", "pack", "derived"];
    let Some(kind) = n.kind.as_deref() else { return Vec::new() };
    if RESERVED.contains(&kind) || graph.scratch_of(n).is_some() {
        return Vec::new();
    }
    let mut need = BTreeSet::new();
    for m in names.candidates(Kind::Type, kind, None).into_iter().filter_map(|d| graph.get(&d.id)) {
        if let Some(list) = m.fields.get("fields").and_then(|v| v.as_array()) {
            for d in list {
                if d.get("required").and_then(|v| v.as_bool()) == Some(true) {
                    if let Some(name) = d.get("name").and_then(|v| v.as_str()) {
                        need.insert(name.to_string());
                    }
                }
            }
        }
    }
    need.into_iter()
        .filter(|f| {
            if f == "body" {
                n.body.trim().is_empty()
            } else {
                match n.fields.get(f) {
                    None => true,
                    Some(v) => v.is_null() || v.as_str() == Some(""),
                }
            }
        })
        .collect()
}

/// `type->to` for each link whose link-type definition lists `from` / `to` node types that the link's ends are not among.
fn endpoint_mismatches(graph: &Graph, names: &Names, n: &crate::model::Node) -> Vec<String> {
    const RESERVED: [&str; 7] = ["plane", "rule", "group", "type", "link-type", "pack", "derived"];
    let Some(kind) = n.kind.as_deref() else { return Vec::new() };
    if RESERVED.contains(&kind) || graph.scratch_of(n).is_some() {
        return Vec::new();
    }
    let mut out = BTreeSet::new();
    for l in &n.links {
        let Some(t) = graph.get(&l.to) else { continue };
        // each list is read from the namespace of the link-type that holds it
        let (mut from, mut to): (Vec<(&str, Option<&str>)>, Vec<(&str, Option<&str>)>) = (Vec::new(), Vec::new());
        for m in names.candidates(Kind::Link, &l.kind, None).into_iter().filter_map(|d| graph.get(&d.id)) {
            for (key, acc) in [("from", &mut from), ("to", &mut to)] {
                if let Some(list) = m.fields.get(key).and_then(|v| v.as_array()) {
                    acc.extend(list.iter().filter_map(|x| x.as_str()).map(|x| (x, def_ns(&m.id))));
                }
            }
        }
        let among = |actual: &str, pairs: &[(&str, Option<&str>)]| {
            pairs.iter().any(|(want, ns)| names.same_str(Kind::Type, actual, want, *ns))
        };
        let bad_from = !from.is_empty() && !among(kind, &from);
        let bad_to = !to.is_empty() && !among(t.kind.as_deref().unwrap_or(""), &to);
        if bad_from || bad_to {
            out.insert(format!("{}->{}", l.kind, l.to));
        }
    }
    out.into_iter().collect()
}

/// Ids with `--` belong to packs. A node must say the same thing twice: its id prefix and its `fields.pack`.
fn namespace_mismatch(graph: &Graph, n: &crate::model::Node) -> bool {
    let ns = n.id.split_once("--").map(|(a, _)| a);
    let tag = n.fields.get("pack").and_then(|v| v.as_str());
    if n.kind.as_deref() == Some("pack") {
        return match n.fields.get("name").and_then(|v| v.as_str()) {
            Some(name) => ns != Some(pack_namespace(graph, name).as_str()),
            None => false,
        };
    }
    match (ns, tag) {
        (None, None) => false,
        (Some(ns), Some(tag)) => ns != pack_namespace(graph, tag.split('@').next().unwrap_or("")),
        _ => true,
    }
}

/// Check the structure of a graph: parents, links, planes, types and namespaces.
pub fn check(graph: &Graph) -> Structure {
    let names = Names::new(graph);
    let planes: BTreeSet<&str> =
        graph.nodes().filter(|n| n.kind.as_deref() == Some("plane")).map(|n| n.id.as_str()).collect();
    let mut out = Structure::default();

    for n in graph.nodes() {
        let mut err = |kind: &'static str, reference: Option<String>| {
            out.errors.push(Finding { node: n.id.clone(), kind, reference });
        };

        if n.kind.as_deref().unwrap_or("").is_empty() {
            err("missing_type", None);
        }
        match n.parent.as_deref() {
            Some(p) if graph.get(p).is_none() => err("missing_parent", Some(p.to_string())),
            _ if reaches_cycle(graph, &n.id) => err("parent_cycle", None),
            _ => {}
        }
        let mut seen = BTreeSet::new();
        for l in &n.links {
            if graph.get(&l.to).is_none() {
                err("missing_link_target", Some(format!("{}->{}", l.kind, l.to)));
            }
            let key = l.key.as_deref().unwrap_or("");
            if !seen.insert((l.kind.as_str(), l.to.as_str(), key)) {
                let suffix = if key.is_empty() { String::new() } else { format!("#{key}") };
                err("duplicate_link", Some(format!("{}->{}{}", l.kind, l.to, suffix)));
            }
        }
        // a scratch is a sandbox: links and parents may not cross its edge, in either direction
        let here = graph.scratch_of(n);
        if let Some(p) = n.parent.as_deref().and_then(|p| graph.get(p)) {
            if graph.scratch_of(p) != here {
                err("scratch_boundary", Some(format!("parent->{}", p.id)));
            }
        }
        for l in &n.links {
            if let Some(t) = graph.get(&l.to) {
                if graph.scratch_of(t) != here {
                    err("scratch_boundary", Some(format!("{}->{}", l.kind, l.to)));
                }
            }
        }
        if namespace_mismatch(graph, n) {
            out.warnings.push(Finding { node: n.id.clone(), kind: "namespace_mismatch", reference: None });
        }
        // a name someone wrote (a title that is not just the id) is at most TITLE_MAX characters
        let len = n.title.chars().count();
        if n.title != n.id && len > TITLE_MAX {
            out.warnings.push(Finding { node: n.id.clone(), kind: "title_too_long", reference: Some(len.to_string()) });
        }
        if let Some(name) = duplicate_definition(graph, n) {
            out.warnings.push(Finding { node: n.id.clone(), kind: "duplicate_definition", reference: Some(name) });
        }
        // a pack's own rule, derived value or link type that uses a bare name two packs define at the same distance
        if matches!(n.kind.as_deref(), Some("rule" | "derived" | "link-type")) {
            if let Some(ns) = def_ns(&n.id) {
                let mut seen = BTreeSet::new();
                for (kind, name) in names_in(n) {
                    if names.ambiguous(kind, &name, Some(ns)) && seen.insert((kind == Kind::Type, name.clone())) {
                        let finding = if kind == Kind::Type { "ambiguous_type" } else { "ambiguous_link_type" };
                        out.warnings.push(Finding { node: n.id.clone(), kind: finding, reference: Some(name) });
                    }
                }
            }
        }
        // a node whose own type, or one of whose link types, is a bare name that two packs define
        if !n.kind.as_deref().is_some_and(|k| crate::rules::RESERVED.contains(&k)) && graph.scratch_of(n).is_none() {
            if let Some(t) = n.kind.as_deref().filter(|t| names.ambiguous(Kind::Type, t, None)) {
                out.warnings.push(Finding {
                    node: n.id.clone(),
                    kind: "ambiguous_type",
                    reference: Some(t.to_string()),
                });
            }
            let links: BTreeSet<&str> =
                n.links.iter().map(|l| l.kind.as_str()).filter(|k| names.ambiguous(Kind::Link, k, None)).collect();
            for lt in links {
                out.warnings.push(Finding {
                    node: n.id.clone(),
                    kind: "ambiguous_link_type",
                    reference: Some(lt.to_string()),
                });
            }
        }
        for r in endpoint_mismatches(graph, &names, n) {
            out.warnings.push(Finding { node: n.id.clone(), kind: "link_endpoint_mismatch", reference: Some(r) });
        }
        for f in missing_required(graph, &names, n) {
            out.warnings.push(Finding { node: n.id.clone(), kind: "missing_required_field", reference: Some(f) });
        }
        if let Some(p) = n.plane.as_deref() {
            if !planes.contains(p) {
                out.warnings.push(Finding {
                    node: n.id.clone(),
                    kind: "undeclared_plane",
                    reference: Some(p.to_string()),
                });
            }
        }
    }

    // Same ordering as the spec: (node, kind, ref).
    let order = |f: &Finding| (f.node.clone(), f.kind, f.reference.clone().unwrap_or_default());
    out.errors.sort_by_key(order);
    out.warnings.sort_by_key(order);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load;

    fn graph(files: &[(&str, &str)]) -> Graph {
        let v = files.iter().map(|(p, t)| (p.to_string(), format!("---\n{t}\n---\n"))).collect();
        load(v).0
    }

    fn kinds(s: &[Finding]) -> Vec<(&str, &'static str)> {
        s.iter().map(|f| (f.node.as_str(), f.kind)).collect()
    }

    #[test]
    fn a_required_body_is_missing_while_the_body_is_blank() {
        let def =
            "---\nid: ty\ntype: type\nfields: {name: note, fields: [{name: body, kind: text, required: true}]}\n---\n";
        let files = |a: &str, b: &str| -> Graph {
            let v = [
                ("ty.md", def.to_string()),
                ("a.md", format!("---\nid: a\ntype: note\n---\n{a}")),
                ("b.md", format!("---\nid: b\ntype: note\n---\n{b}")),
            ];
            load(v.iter().map(|(p, t)| (p.to_string(), t.clone())).collect()).0
        };
        let s = check(&files("", "Some words.\n"));
        let miss: Vec<_> = s
            .warnings
            .iter()
            .filter(|f| f.kind == "missing_required_field")
            .map(|f| (f.node.as_str(), f.reference.as_deref()))
            .collect();
        assert_eq!(miss, [("a", Some("body"))]);
    }

    #[test]
    fn clean_graph_has_no_findings() {
        let s = check(&graph(&[("a.md", "id: a\ntype: t")]));
        assert!(s.errors.is_empty() && s.warnings.is_empty());
    }

    #[test]
    fn cycle_is_reported_on_every_node_that_reaches_it() {
        let g = graph(&[
            ("a.md", "id: a\ntype: t\nparent: b"),
            ("b.md", "id: b\ntype: t\nparent: a"),
            ("c.md", "id: c\ntype: t\nparent: a"),
        ]);
        assert_eq!(kinds(&check(&g).errors), [("a", "parent_cycle"), ("b", "parent_cycle"), ("c", "parent_cycle")]);
    }

    #[test]
    fn missing_parent_is_only_on_the_node_that_names_it() {
        let g = graph(&[("a.md", "id: a\ntype: t\nparent: b"), ("b.md", "id: b\ntype: t\nparent: gone")]);
        assert_eq!(kinds(&check(&g).errors), [("b", "missing_parent")]);
    }
}
