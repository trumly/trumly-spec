//! Name resolution: which definition a type or link type name means. Mirrors `candidates` and `same_name` in reference/core.py and
//! section 4a of spec/FORMAT.md.
//!
//! A name is bare (`service`) or qualified by a pack namespace (`k8s:service`). A definition belongs to the namespace in its id
//! (`k8s--type-service`). Inside a pack a bare name means the nearest level that has a definition: the pack's own, then the packs it
//! requires, then the packs they require. Several namespaces at the winning level, or in the whole project, leave it ambiguous.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

use serde_json::Value;

use crate::model::Graph;
use crate::rules::json_eq;

/// Which kind of definition a name is looked up among.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A node type (`type` nodes).
    Type,
    /// A link type (`link-type` nodes).
    Link,
}

/// Whether a link kind is `name`, bare or qualified (`conv:describes`): for the few links the core itself follows.
pub fn kind_is(kind: &str, name: &str) -> bool {
    kind == name || kind.split_once(':').is_some_and(|(ns, n)| !ns.is_empty() && n == name)
}

/// The namespace a node id belongs to: the part before `--`, if it has one.
pub fn def_ns(id: &str) -> Option<&str> {
    id.split_once("--").map(|(ns, _)| ns)
}

/// One `type` or `link-type` node that defines a name.
#[derive(Debug)]
pub struct Def {
    /// The definition node's id.
    pub id: String,
    /// Its namespace, if its id has one.
    pub ns: Option<String>,
}

/// Every definition of every name, and which packs require or work with which, read once from a graph.
#[derive(Debug, Default)]
pub struct Names {
    types: BTreeMap<String, Vec<Def>>,
    links: BTreeMap<String, Vec<Def>>,
    requires: BTreeMap<String, BTreeSet<String>>,
}

impl Names {
    /// Index the definitions and the pack `requires` of a graph.
    pub fn new(graph: &Graph) -> Names {
        let mut out = Names::default();
        // a pack `optional` counts as a requirement when the project has it, so those wait until every pack is known
        let mut optional: Vec<(String, Vec<String>)> = Vec::new();
        let present: BTreeSet<&str> = graph
            .nodes()
            .filter(|n| n.kind.as_deref() == Some("pack"))
            .filter_map(|n| n.fields.get("name").and_then(Value::as_str))
            .collect();
        for n in graph.nodes() {
            let name = n.fields.get("name").and_then(Value::as_str);
            let def = || Def { id: n.id.clone(), ns: def_ns(&n.id).map(str::to_string) };
            match (n.kind.as_deref(), name) {
                (Some("type"), Some(name)) => out.types.entry(name.to_string()).or_default().push(def()),
                (Some("link-type"), Some(name)) => out.links.entry(name.to_string()).or_default().push(def()),
                (Some("pack"), _) => {
                    if let Some(ns) = def_ns(&n.id) {
                        let wanted = n.fields.get("requires").and_then(Value::as_array).into_iter().flatten();
                        let set: BTreeSet<String> = wanted
                            .filter_map(Value::as_str)
                            .map(|r| crate::structure::pack_namespace(graph, r.split('@').next().unwrap_or("")))
                            .collect();
                        out.requires.entry(ns.to_string()).or_default().extend(set);
                        let works_with = n.fields.get("optional").and_then(Value::as_array).into_iter().flatten();
                        let wanted: Vec<String> = works_with
                            .filter_map(Value::as_str)
                            .map(|r| r.split('@').next().unwrap_or("").to_string())
                            .filter(|r| present.contains(r.as_str()))
                            .collect();
                        optional.push((ns.to_string(), wanted));
                    }
                }
                _ => {}
            }
        }
        for (ns, wanted) in optional {
            for name in wanted {
                let other = crate::structure::pack_namespace(graph, &name);
                out.requires.entry(ns.clone()).or_default().insert(other);
            }
        }
        out
    }

    /// Namespaces by distance from pack `ctx`: itself, the packs it requires or works with (`optional`, when the project has them), then theirs. Each appears once, nearest.
    fn levels(&self, ctx: &str) -> Vec<BTreeSet<String>> {
        let mut out = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut cur: BTreeSet<String> = BTreeSet::from([ctx.to_string()]);
        while !cur.is_empty() {
            seen.extend(cur.iter().cloned());
            let mut next = BTreeSet::new();
            for ns in &cur {
                for q in self.requires.get(ns).into_iter().flatten() {
                    if !seen.contains(q) {
                        next.insert(q.clone());
                    }
                }
            }
            out.push(std::mem::replace(&mut cur, next));
        }
        out
    }

    /// The definitions `name` can mean, read from pack namespace `ctx` (`None` for a node, a link, or a definition outside a pack).
    pub fn candidates(&self, kind: Kind, name: &str, ctx: Option<&str>) -> Vec<&Def> {
        let map = match kind {
            Kind::Type => &self.types,
            Kind::Link => &self.links,
        };
        if let Some((ns, bare)) = name.split_once(':') {
            return map.get(bare).map_or_else(Vec::new, |v| v.iter().filter(|d| d.ns.as_deref() == Some(ns)).collect());
        }
        let Some(every) = map.get(name) else { return Vec::new() };
        if let Some(ctx) = ctx.filter(|_| every.len() > 1) {
            for level in self.levels(ctx) {
                let near: Vec<&Def> =
                    every.iter().filter(|d| d.ns.as_ref().is_some_and(|n| level.contains(n))).collect();
                if !near.is_empty() {
                    return near;
                }
            }
        }
        every.iter().collect()
    }

    /// True when a bare `name`, read from `ctx`, still has definitions in two or more namespaces.
    pub fn ambiguous(&self, kind: Kind, name: &str, ctx: Option<&str>) -> bool {
        if name.contains(':') {
            return false;
        }
        let spaces: BTreeSet<Option<&str>> = self.candidates(kind, name, ctx).iter().map(|d| d.ns.as_deref()).collect();
        spaces.len() > 1
    }

    /// Do `actual` (what a node or link says) and `want` (what a rule, link type or template says, read from `ctx`) denote the same
    /// thing? By definition when both resolve, otherwise by plain equality.
    pub fn same_str(&self, kind: Kind, actual: &str, want: &str, ctx: Option<&str>) -> bool {
        let a = self.candidates(kind, actual, None);
        let w = self.candidates(kind, want, ctx);
        if !a.is_empty() && !w.is_empty() {
            return a.iter().any(|x| w.iter().any(|y| x.id == y.id));
        }
        actual == want
    }

    /// `same_str` for JSON values: strings resolve, anything else is compared by `json_eq`.
    pub fn same(&self, kind: Kind, actual: &Value, want: &Value, ctx: Option<&str>) -> bool {
        match (actual.as_str(), want.as_str()) {
            (Some(a), Some(w)) => self.same_str(kind, a, w, ctx),
            _ => json_eq(actual, want),
        }
    }
}

/// A graph, its names, and the namespace of the pack whose rule, derived field or link type is being read. Derefs to the graph, so
/// code that only needs nodes can use it as one.
#[derive(Clone, Copy)]
pub struct Scope<'a> {
    graph: &'a Graph,
    names: &'a Names,
    ctx: Option<&'a str>,
}

impl<'a> Scope<'a> {
    /// A scope reading names from namespace `ctx`.
    pub fn new(graph: &'a Graph, names: &'a Names, ctx: Option<&'a str>) -> Scope<'a> {
        Scope { graph, names, ctx }
    }

    /// The same graph, read from another namespace.
    pub fn at(&self, ctx: Option<&'a str>) -> Scope<'a> {
        Scope { ctx, ..*self }
    }

    /// The names of the graph.
    pub fn names(&self) -> &'a Names {
        self.names
    }

    /// Does a node's type (a JSON value) mean the type a rule names?
    pub fn same_type(&self, actual: &Value, want: &Value) -> bool {
        self.names.same(Kind::Type, actual, want, self.ctx)
    }

    /// `same_type` for two strings.
    pub fn same_type_str(&self, actual: &str, want: &str) -> bool {
        self.names.same_str(Kind::Type, actual, want, self.ctx)
    }

    /// Does a link's type mean the link type a rule names?
    pub fn same_link(&self, actual: &str, want: &str) -> bool {
        self.names.same_str(Kind::Link, actual, want, self.ctx)
    }
}

impl Deref for Scope<'_> {
    type Target = Graph;
    fn deref(&self) -> &Graph {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::kind_is;

    #[test]
    fn a_link_kind_matches_its_bare_or_qualified_name() {
        assert!(kind_is("describes", "describes") && kind_is("conv:describes", "describes"));
        assert!(
            !kind_is("describes-not", "describes")
                && !kind_is(":describes", "describes")
                && !kind_is("conv:other", "describes")
        );
    }
}
