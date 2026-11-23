//! Derived values: read-only values a node has but nobody types in, worked out from a template over the node's own
//! fields and the nodes it links to. Mirrors section 7a of the spec and `derive` in reference/core.py.
//!
//! A `derived` node defines one: its `name`, its `template` and an `applies_to` filter (the same filter rules use).
//! The values are never written to a file; they are computed from the graph every time it is read.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::model::{Graph, Node};
use crate::names::{Kind, Names, Scope, def_ns};
use crate::rules::{self, Inbound, RESERVED};

/// The most values one derived name gives a node. Several placeholders that each reach several nodes multiply; this stops that.
pub const MAX_VALUES: usize = 64;
/// The most placeholders a template may have, and (below) the most links each may follow. A template past either is unusable
/// (`missing: ["template"]`), so one node cannot make the engine search without end.
pub const MAX_PLACEHOLDERS: usize = 32;
/// The most links one placeholder may follow before it reads a field. The longest in any shipped pack is four.
pub const MAX_HOPS: usize = 5;
/// The most characters a derived value may have. A template that would build a longer one is unusable (`missing: ["template"]`).
pub const MAX_VALUE_CHARS: usize = 1024;

/// What one derived name came to for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Derived {
    /// Every placeholder resolved: the values, one per combination, without repeats.
    Values(Vec<String>),
    /// At least one placeholder reached nothing: the placeholders (as written, without braces) that did not resolve.
    Missing(Vec<String>),
    /// The node says what its name really is (`overrides`), so that replaces the template. `computed` is what the template
    /// gave, so a reader can show that the node is an exception (empty when the template could not be resolved).
    Overridden {
        /// the values the node says it has, which replace the template's
        values: Vec<String>,
        /// the values the template gave
        computed: Vec<String>,
    },
}

impl Derived {
    /// `{"values": [...]}`, `{"missing": [...]}` or `{"values": [...], "override": true, "computed": [...]}`.
    pub fn to_json(&self) -> Value {
        match self {
            Derived::Values(v) => json!({ "values": v }),
            Derived::Missing(m) => json!({ "missing": m }),
            Derived::Overridden { values, computed } => {
                json!({ "values": values, "override": true, "computed": computed })
            }
        }
    }
}

/// Every node's derived values, by node id and then derived name. Nodes with none are left out.
pub type DerivedMap = BTreeMap<String, BTreeMap<String, Derived>>;

enum Part {
    Lit(String),
    Ph(String),
}

/// Split a template into text and `{placeholders}`. `\{`, `\}` and `\\` are literal characters. `None` if a brace is
/// unbalanced or a placeholder is empty.
fn parse_template(t: &str) -> Option<Vec<Part>> {
    let mut parts = Vec::new();
    let mut lit = String::new();
    let mut placeholders = 0;
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.peek() {
                Some('{' | '}' | '\\') => lit.push(chars.next()?),
                _ => lit.push('\\'),
            },
            '{' => {
                let mut ph = String::new();
                let mut closed = false;
                for d in chars.by_ref() {
                    if d == '}' {
                        closed = true;
                        break;
                    }
                    if d == '{' {
                        return None;
                    }
                    ph.push(d);
                }
                placeholders += 1;
                if !closed || ph.is_empty() || placeholders > MAX_PLACEHOLDERS || ph.matches('/').count() > MAX_HOPS {
                    return None;
                }
                if !lit.is_empty() {
                    parts.push(Part::Lit(std::mem::take(&mut lit)));
                }
                parts.push(Part::Ph(ph));
            }
            '}' => return None,
            _ => lit.push(c),
        }
    }
    if !lit.is_empty() {
        parts.push(Part::Lit(lit));
    }
    Some(parts)
}

/// The link type names a template follows (every hop but `@type`, without the `^` that marks a backwards one).
pub(crate) fn template_links(template: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in parse_template(template).into_iter().flatten() {
        if let Part::Ph(ph) = part {
            let mut segs: Vec<&str> = ph.split('/').collect();
            segs.pop(); // the last segment is the field read, not a hop
            for hop in segs.into_iter().filter(|h| !h.is_empty() && *h != "@type") {
                out.push(hop.strip_prefix('^').unwrap_or(hop).to_string());
            }
        }
    }
    out
}

struct Def<'a> {
    name: &'a str,
    parts: Option<Vec<Part>>,
    filter: Map<String, Value>,
    /// the namespace of the pack the definition is in: its template's link names are read from there
    ns: Option<&'a str>,
    /// id 3, type 2, plane 1, anything else 0: the most specific definition for a name wins
    spec: u8,
}

fn specificity(f: &Map<String, Value>) -> u8 {
    if f.contains_key("id") {
        3
    } else if f.contains_key("type") {
        2
    } else if f.contains_key("plane") {
        1
    } else {
        0
    }
}

struct Ctx<'a> {
    graph: &'a Graph,
    inbound: Inbound<'a>,
    /// every type and link type definition, to resolve the names a template uses
    names: &'a Names,
}

/// The nodes a run of hops reaches from `node`, in id order without repeats. A hop is a link type (followed outwards),
/// `^link-type` (followed inwards) or `@type` (the type definition of each node reached so far, at any hop).
fn reach<'a>(cx: &Ctx<'a>, ns: Option<&str>, node: &'a Node, hops: &[&str]) -> Vec<&'a Node> {
    let mut cur: BTreeMap<&str, &Node> = BTreeMap::new();
    cur.insert(node.id.as_str(), node);
    for hop in hops {
        let mut next: BTreeMap<&str, &Node> = BTreeMap::new();
        for n in cur.values() {
            if *hop == "@type" {
                // the type definition of a node: its name read project-wide, the first by id when two packs define it
                let found = n
                    .kind
                    .as_deref()
                    .and_then(|k| cx.names.candidates(Kind::Type, k, None).first().map(|d| d.id.clone()));
                if let Some(t) = found.and_then(|id| cx.graph.get(&id)) {
                    next.insert(t.id.as_str(), t);
                }
            } else if let Some(name) = hop.strip_prefix('^') {
                for &(src, l) in cx.inbound.get(n.id.as_str()).into_iter().flatten() {
                    if cx.names.same_str(Kind::Link, &l.kind, name, ns) {
                        next.insert(src.id.as_str(), src);
                    }
                }
            } else {
                for l in &n.links {
                    if cx.names.same_str(Kind::Link, &l.kind, hop, ns) {
                        if let Some(t) = cx.graph.get(&l.to) {
                            next.insert(t.id.as_str(), t);
                        }
                    }
                }
            }
        }
        cur = next;
    }
    cur.into_values().collect()
}

/// A value as text: strings as they are, numbers as JSON writes them, booleans as `true` / `false`. Anything else has none.
fn text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// What a field path holds on `n`, as text. `id`, `title` and `@type` (its type name) read the node's own when no field has that name.
fn read(n: &Node, path: &str) -> Vec<String> {
    if !n.fields.contains_key(path) {
        if path == "id" {
            return vec![n.id.clone()];
        }
        if path == "title" {
            return vec![n.title.clone()];
        }
        if path == "@type" {
            return n.kind.clone().into_iter().collect();
        }
    }
    rules::resolve(|k| n.fields.get(k), path).into_iter().flatten().flatten().filter_map(text).collect()
}

/// The values one placeholder gives for `node`. Empty means it reached nothing.
fn placeholder<'a>(cx: &Ctx<'a>, ns: Option<&str>, node: &'a Node, ph: &str) -> Vec<String> {
    let segs: Vec<&str> = ph.split('/').collect();
    if segs.iter().any(|s| s.is_empty()) {
        return Vec::new();
    }
    let Some((last, hops)) = segs.split_last() else { return Vec::new() };
    reach(cx, ns, node, hops).into_iter().flat_map(|n| read(n, last)).collect()
}

fn evaluate<'a>(cx: &Ctx<'a>, node: &'a Node, def: &Def) -> Derived {
    let Some(parts) = def.parts.as_ref().filter(|p| !p.is_empty()) else {
        return Derived::Missing(vec!["template".to_string()]);
    };
    let mut results = vec![String::new()];
    let mut missing: Vec<String> = Vec::new();
    for p in parts {
        match p {
            Part::Lit(s) => {
                for r in &mut results {
                    if r.chars().count() + s.chars().count() > MAX_VALUE_CHARS {
                        return Derived::Missing(vec!["template".to_string()]);
                    }
                    r.push_str(s);
                }
            }
            Part::Ph(ph) => {
                let vals = placeholder(cx, def.ns, node, ph);
                if vals.is_empty() {
                    if !missing.contains(ph) {
                        missing.push(ph.clone());
                    }
                    continue;
                }
                let mut next = Vec::new();
                'outer: for r in &results {
                    for v in &vals {
                        if next.len() >= MAX_VALUES {
                            break 'outer;
                        }
                        if r.chars().count() + v.chars().count() > MAX_VALUE_CHARS {
                            return Derived::Missing(vec!["template".to_string()]);
                        }
                        next.push(format!("{r}{v}"));
                    }
                }
                results = next;
            }
        }
    }
    if !missing.is_empty() {
        return Derived::Missing(missing);
    }
    let mut seen: Vec<String> = Vec::new();
    for r in results {
        if !seen.contains(&r) {
            seen.push(r);
        }
    }
    Derived::Values(seen)
}

/// What the node says its name is: `overrides: {<derived name>: "value"}` (or a list of values). Anything else is ignored.
fn overridden(node: &Node, name: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let one = |v: &Value, out: &mut Vec<String>| {
        if let Some(t) = v.as_str().filter(|t| !t.is_empty()) {
            if !out.iter().any(|o| o == t) && out.len() < MAX_VALUES {
                out.push(t.to_string());
            }
        }
    };
    match node.fields.get("overrides").and_then(Value::as_object).and_then(|m| m.get(name)) {
        Some(Value::Array(a)) => a.iter().for_each(|v| one(v, &mut out)),
        Some(v) => one(v, &mut out),
        None => {}
    }
    out
}

/// Work out every node's derived values (by `field-name`, else `name`). Reserved nodes and nodes in a scratch have none. For each name, the most specific
/// definition that applies wins (an `id` filter, then `type`, then `plane`, then any other); a tie goes to the lower definition id.
pub fn derive_all(graph: &Graph) -> DerivedMap {
    let defs: Vec<Def> = graph
        .nodes()
        .filter(|n| n.kind.as_deref() == Some("derived") && n.fields.get("enabled") != Some(&Value::Bool(false)))
        .filter_map(|n| {
            // `name` is the definition's own name; `field-name` (when set) is what the value is called on a node, so several
            // definitions, each for its own kind of node, can give one field
            let own = n.fields.get("name")?.as_str()?;
            let name = n.fields.get("field-name").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or(own);
            let template = n.fields.get("template")?.as_str()?;
            if own.is_empty() {
                return None;
            }
            let filter = n.fields.get("applies_to").and_then(Value::as_object).cloned().unwrap_or_default();
            Some(Def { name, parts: parse_template(template), spec: specificity(&filter), filter, ns: def_ns(&n.id) })
        })
        .collect();
    let mut out = DerivedMap::new();
    if defs.is_empty() {
        return out;
    }
    let names = Names::new(graph);
    let cx = Ctx { graph, inbound: rules::build_inbound(graph), names: &names };
    for n in graph.nodes() {
        if n.kind.as_deref().is_some_and(|k| RESERVED.contains(&k)) || graph.scratch_of(n).is_some() {
            continue;
        }
        let mut best: BTreeMap<&str, &Def> = BTreeMap::new();
        for d in &defs {
            if !rules::matches(&Scope::new(graph, &names, d.ns), n, &d.filter) {
                continue;
            }
            match best.get(d.name) {
                Some(b) if b.spec >= d.spec => {}
                _ => {
                    best.insert(d.name, d);
                }
            }
        }
        if best.is_empty() {
            continue;
        }
        let row: BTreeMap<String, Derived> = best
            .into_iter()
            .map(|(name, d)| {
                let computed = evaluate(&cx, n, d);
                let values = overridden(n, name);
                let result = if values.is_empty() {
                    computed
                } else {
                    let computed = match computed {
                        Derived::Values(v) => v,
                        _ => Vec::new(),
                    };
                    Derived::Overridden { values, computed }
                };
                (name.to_string(), result)
            })
            .collect();
        out.insert(n.id.clone(), row);
    }
    out
}

/// One node's derived values as JSON (`{name: {"values": [...]} | {"missing": [...]}}`).
pub fn row_to_json(row: &BTreeMap<String, Derived>) -> Value {
    Value::Object(row.iter().map(|(k, v)| (k.clone(), v.to_json())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::check;

    fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(id, body)| (format!("{id}.md"), format!("---\nid: {id}\n{body}\n---\n"))).collect()
    }

    fn values(map: &DerivedMap, id: &str, name: &str) -> Vec<String> {
        match &map[id][name] {
            Derived::Values(v) => v.clone(),
            Derived::Missing(m) => panic!("missing {m:?}"),
            Derived::Overridden { values, .. } => values.clone(),
        }
    }

    #[test]
    fn template_parts_and_escapes() {
        let p = parse_template(r"a{b/c}\{d\}").unwrap();
        assert_eq!(p.len(), 3);
        assert!(parse_template("{").is_none());
        assert!(parse_template("}").is_none());
        assert!(parse_template("{}").is_none());
        assert!(parse_template("{a{b}}").is_none());
    }

    #[test]
    fn fields_links_and_the_type_token() {
        let r = check(files(&[
            (
                "d-name",
                "type: derived\nfields:\n  name: name\n  template: '{@type/^stands-for/token}-{workload}-{runs-in/token}-{runs-in/in-region/token}'",
            ),
            ("t-db", "type: type\nfields:\n  name: db"),
            ("tok-psql", "type: token\nfields:\n  token: psql\nlinks:\n  - {type: stands-for, to: t-db}"),
            ("env-prod", "type: env\nfields:\n  token: prod\nlinks:\n  - {type: in-region, to: reg-uks}"),
            ("reg-uks", "type: region\nfields:\n  token: uks"),
            ("ops", "type: db\nfields:\n  workload: ops\nlinks:\n  - {type: runs-in, to: env-prod}"),
        ]));
        let m = derive_all(&r.graph);
        assert_eq!(values(&m, "ops", "name"), ["psql-ops-prod-uks"]);
        // definitions are reserved, so they have none
        assert!(!m.contains_key("d-name"));
    }

    #[test]
    fn fan_out_and_missing() {
        let r = check(files(&[
            ("d", "type: derived\nfields:\n  name: name\n  template: '{id}-{runs-in/token}'"),
            ("e1", "type: env\nfields:\n  token: dev"),
            ("e2", "type: env\nfields:\n  token: prod"),
            ("two", "type: db\nlinks:\n  - {type: runs-in, to: e2}\n  - {type: runs-in, to: e1}"),
            ("none", "type: db"),
        ]));
        let m = derive_all(&r.graph);
        assert_eq!(values(&m, "two", "name"), ["two-dev", "two-prod"]);
        assert_eq!(m["none"]["name"], Derived::Missing(vec!["runs-in/token".to_string()]));
    }

    #[test]
    fn most_specific_wins_and_ties_go_to_the_lower_id() {
        let r = check(files(&[
            ("a-all", "type: derived\nfields:\n  name: name\n  template: 'all'"),
            ("b-type", "type: derived\nfields:\n  name: name\n  template: 'type'\n  applies_to: {type: db}"),
            ("c-one", "type: derived\nfields:\n  name: name\n  template: 'one'\n  applies_to: {id: x}"),
            ("c-tie", "type: derived\nfields:\n  name: name\n  template: 'tie'\n  applies_to: {id: x}"),
            ("x", "type: db"),
            ("y", "type: db"),
            ("z", "type: other"),
        ]));
        let m = derive_all(&r.graph);
        assert_eq!(values(&m, "x", "name"), ["one"]);
        assert_eq!(values(&m, "y", "name"), ["type"]);
        assert_eq!(values(&m, "z", "name"), ["all"]);
    }

    #[test]
    fn definitions_with_their_own_names_can_share_a_field_name() {
        let r = check(files(&[
            (
                "d-a",
                "type: derived\nfields:\n  name: a-handle\n  field-name: handle\n  template: 'a-{id}'\n  applies_to: {type: a}",
            ),
            (
                "d-b",
                "type: derived\nfields:\n  name: b-handle\n  field-name: handle\n  template: 'b-{id}'\n  applies_to: {type: b}",
            ),
            ("t-a", "type: type\nfields:\n  name: a"),
            ("t-b", "type: type\nfields:\n  name: b"),
            ("x", "type: a"),
            ("y", "type: b"),
        ]));
        let m = derive_all(&r.graph);
        assert_eq!(values(&m, "x", "handle"), ["a-x"]);
        assert_eq!(values(&m, "y", "handle"), ["b-y"]);
        assert!(!m["x"].contains_key("a-handle"));
    }

    #[test]
    fn an_override_replaces_the_template_and_keeps_what_it_gave() {
        let g = check(files(&[
            ("d", "type: derived\nfields:\n  name: label\n  template: 'n-{id}'"),
            ("plain", "type: thing"),
            ("odd", "type: thing\nfields:\n  overrides:\n    label: legacy-odd"),
            ("two", "type: thing\nfields:\n  overrides:\n    label: [b, a, b, '']"),
            ("junk", "type: thing\nfields:\n  overrides:\n    label: 7\n    other: x"),
        ]))
        .derived;
        assert_eq!(values(&g, "plain", "label"), ["n-plain"]);
        assert_eq!(values(&g, "odd", "label"), ["legacy-odd"]);
        assert_eq!(
            g["odd"]["label"].to_json(),
            serde_json::json!({"values": ["legacy-odd"], "override": true, "computed": ["n-odd"]})
        );
        assert_eq!(values(&g, "two", "label"), ["b", "a"]);
        assert_eq!(values(&g, "junk", "label"), ["n-junk"]); // not a string: the template stands
    }
}
