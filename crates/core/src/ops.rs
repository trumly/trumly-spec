//! Edits that must mean the same thing in every client (the editor, the CLI, an agent over MCP): renaming a node and
//! renaming a type or link type. Each takes the project and returns an `Edit`: the nodes to write (new or changed
//! files) and the ids whose files go. Nothing here touches the disk.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::model::{Graph, Node};

/// What a rename changes: write each node in `write` (by its id) and remove the files of the ids in `delete`.
#[derive(Debug, Default, PartialEq)]
pub struct Edit {
    /// Nodes to write, each under its id.
    pub write: Vec<Node>,
    /// Ids whose files should be removed.
    pub delete: Vec<String>,
    /// (old id, new id) for each node that was renamed, so a client can put the new file beside the old one
    pub renamed: Vec<(String, String)>,
}

/// Why `to` cannot be the id of the node now called `from`, or `None`.
pub fn id_problem(graph: &Graph, from: &str, to: &str) -> Option<String> {
    if to.is_empty() {
        return Some("An id cannot be empty.".into());
    }
    if to != to.trim() {
        return Some("An id cannot start or end with a space.".into());
    }
    if to.contains(['/', '\\']) {
        return Some("An id cannot contain / or \\.".into());
    }
    if to.starts_with('.') {
        return Some("An id cannot start with a dot.".into());
    }
    if to.chars().any(char::is_control) {
        return Some("An id cannot contain control characters.".into());
    }
    if to.len() > 200 {
        return Some("An id is at most 200 characters.".into());
    }
    if to != from {
        if graph.get(to).is_some() {
            return Some(format!("“{to}” already exists."));
        }
        let lower = to.to_lowercase();
        if graph.nodes().any(|n| n.id != from && n.id.to_lowercase() == lower) {
            return Some(format!(
                "“{to}” only differs by capitals from an existing id; they would be one file on macOS and Windows."
            ));
        }
    }
    None
}

/// The working copy an edit is built on.
struct Work {
    nodes: BTreeMap<String, Node>,
    changed: BTreeSet<String>,
    deleted: BTreeSet<String>,
    renamed: Vec<(String, String)>,
}

impl Work {
    fn new(graph: &Graph) -> Work {
        Work {
            nodes: graph.nodes().map(|n| (n.id.clone(), n.clone())).collect(),
            changed: BTreeSet::new(),
            deleted: BTreeSet::new(),
            renamed: Vec::new(),
        }
    }

    fn finish(self) -> Edit {
        let write = self.changed.iter().filter_map(|id| self.nodes.get(id).cloned()).collect();
        Edit { write, delete: self.deleted.into_iter().collect(), renamed: self.renamed }
    }

    /// Give a node a new id and fix everything that points at it: parents, links, plane membership, rule filters.
    fn rename(&mut self, from: &str, to: &str) {
        if from == to {
            return;
        }
        let Some(mut node) = self.nodes.remove(from) else { return };
        self.renamed.push((from.to_string(), to.to_string()));
        let is_plane = node.kind.as_deref() == Some("plane");
        node.id = to.to_string();
        self.nodes.insert(to.to_string(), node);
        self.changed.remove(from);
        self.changed.insert(to.to_string());
        if !self.deleted.remove(to) {
            self.deleted.insert(from.to_string());
        }
        for (id, m) in self.nodes.iter_mut() {
            let mut hit = false;
            if m.parent.as_deref() == Some(from) {
                m.parent = Some(to.to_string());
                hit = true;
            }
            if is_plane && m.plane.as_deref() == Some(from) {
                m.plane = Some(to.to_string());
                hit = true;
            }
            for l in m.links.iter_mut().filter(|l| l.to == from) {
                l.to = to.to_string();
                hit = true;
            }
            if m.kind.as_deref() == Some("rule") {
                for key in ["applies_to", "check"] {
                    if let Some(v) = m.fields.get_mut(key) {
                        hit |= rewrite_ids(v, from, to, is_plane, "");
                    }
                }
            }
            if hit {
                self.changed.insert(id.clone());
            }
        }
    }

    fn touch(&mut self, id: &str) {
        self.changed.insert(id.to_string());
    }
}

const PLANE_KEYS: [&str; 3] = ["plane", "node_plane", "to_plane"];

/// Rewrite id references inside a rule's filter or check: `under` names any node, plane keys name planes.
fn rewrite_ids(v: &mut Value, from: &str, to: &str, is_plane: bool, key: &str) -> bool {
    match v {
        Value::String(s) => {
            if s == from && (key == "under" || (is_plane && PLANE_KEYS.contains(&key))) {
                *s = to.to_string();
                true
            } else {
                false
            }
        }
        Value::Array(a) => a.iter_mut().fold(false, |h, x| rewrite_ids(x, from, to, is_plane, key) | h),
        Value::Object(o) => o.iter_mut().fold(false, |h, (k, x)| rewrite_ids(x, from, to, is_plane, k) | h),
        _ => false,
    }
}

/// Rename a node (its file, and every link, parent, plane membership and rule filter that names it).
pub fn rename_node(graph: &Graph, from: &str, to: &str) -> Result<Edit, String> {
    let to = to.trim();
    if let Some(p) = id_problem(graph, from, to) {
        return Err(p);
    }
    if graph.get(from).is_none() {
        return Err(format!("“{from}” does not exist."));
    }
    let mut w = Work::new(graph);
    w.rename(from, to);
    Ok(w.finish())
}

/// Replace `old` by `new` under any of `keys`, anywhere inside `v`.
fn rename_in(v: &mut Value, keys: &[&str], old: &str, new: &str) -> bool {
    match v {
        Value::Array(a) => a.iter_mut().fold(false, |h, x| rename_in(x, keys, old, new) | h),
        Value::Object(o) => o.iter_mut().fold(false, |h, (k, x)| {
            if keys.contains(&k.as_str()) && x.as_str() == Some(old) {
                *x = Value::String(new.to_string());
                true | h
            } else {
                rename_in(x, keys, old, new) | h
            }
        }),
        _ => false,
    }
}

/// The link type names a rule check refers to: `type` on link items and path steps, and `acyclic`.
fn rename_link_in(check: &mut Value, old: &str, new: &str) -> bool {
    let Some(c) = check.as_object_mut() else { return false };
    let mut hit = false;
    for key in ["require_link", "forbid_link", "path", "require_link_fields"] {
        let items: Vec<&mut Value> = match c.get_mut(key) {
            Some(Value::Array(a)) => a.iter_mut().collect(),
            Some(v @ Value::Object(_)) => vec![v],
            _ => vec![],
        };
        for it in items {
            if let Some(t) = it.get_mut("type").filter(|t| t.as_str() == Some(old)) {
                *t = Value::String(new.to_string());
                hit = true;
            }
        }
    }
    if let Some(Value::Array(a)) = c.get_mut("any_of") {
        for it in a.iter_mut() {
            hit |= rename_link_in(it, old, new);
        }
    }
    match c.get_mut("acyclic") {
        Some(a @ Value::String(_)) if a.as_str() == Some(old) => {
            *a = Value::String(new.to_string());
            hit = true;
        }
        Some(Value::Array(a)) => {
            for t in a.iter_mut().filter(|t| t.as_str() == Some(old)) {
                *t = Value::String(new.to_string());
                hit = true;
            }
        }
        _ => {}
    }
    hit
}

/// The id a node of this kind would follow the type rename to: `<ns>--<old>-rest` becomes `<ns>--<new>-rest`.
fn followed_id(id: &str, old: &str, new: &str) -> Option<String> {
    let (ns, local) = match id.split_once("--") {
        Some((ns, local)) => (format!("{ns}--"), local),
        None => (String::new(), id),
    };
    local.strip_prefix(&format!("{old}-")).map(|rest| format!("{ns}{new}-{rest}"))
}

/// Rename a type or a link type, given its definition node, and everything that uses the old name: nodes of the type
/// (or links of the link type), rule filters and checks, and link types whose from/to list the type. With `ids`, nodes
/// of a renamed type whose id follows `<type>-name` get the new type in their id too (references are fixed).
pub fn rename_def(graph: &Graph, def_id: &str, name: &str, ids: bool) -> Result<Edit, String> {
    let def = graph.get(def_id).ok_or(format!("“{def_id}” does not exist."))?;
    let kind = def.kind.clone().unwrap_or_default();
    if kind != "type" && kind != "link-type" {
        return Err(format!("“{def_id}” is not a type or a link type."));
    }
    let old = def.fields.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    let new = if name.trim().is_empty() { def.id.clone() } else { name.trim().to_string() };
    if new == old {
        return Ok(Edit::default());
    }
    if graph.nodes().any(|n| {
        n.id != def_id
            && n.kind.as_deref() == Some(kind.as_str())
            && n.fields.get("name").and_then(Value::as_str) == Some(new.as_str())
    }) {
        return Err(format!(
            "There is already a {} called “{new}”.",
            if kind == "type" { "type" } else { "link type" }
        ));
    }
    let mut w = Work::new(graph);
    {
        let d = w.nodes.get_mut(def_id).expect("it was just read");
        d.fields.insert("name".into(), Value::String(new.clone()));
        d.title = new.clone();
    }
    w.touch(def_id);
    let ids_of = |w: &Work| w.nodes.keys().cloned().collect::<Vec<_>>();
    if kind == "type" {
        let mut moved = Vec::new();
        for id in ids_of(&w) {
            let n = w.nodes.get_mut(&id).expect("listed");
            match n.kind.as_deref() {
                Some(k) if k == old => {
                    n.kind = Some(new.clone());
                    moved.push(id.clone());
                    w.changed.insert(id);
                }
                Some("rule") => {
                    let mut hit = false;
                    if let Some(a) = n.fields.get_mut("applies_to").and_then(Value::as_object_mut) {
                        // `type` is one name, or a list of names (any of)
                        match a.get_mut("type") {
                            Some(Value::String(t)) if *t == old => {
                                *t = new.clone();
                                hit = true;
                            }
                            Some(Value::Array(list)) => {
                                for t in list.iter_mut().filter(|t| t.as_str() == Some(old.as_str())) {
                                    *t = Value::String(new.clone());
                                    hit = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(c) = n.fields.get_mut("check") {
                        hit |= rename_in(c, &["node_type", "to_type", "ancestor_type"], &old, &new);
                    }
                    if hit {
                        w.changed.insert(id);
                    }
                }
                Some("link-type") => {
                    let mut hit = false;
                    for key in ["from", "to"] {
                        if let Some(Value::Array(a)) = n.fields.get_mut(key) {
                            for t in a.iter_mut().filter(|t| t.as_str() == Some(old.as_str())) {
                                *t = Value::String(new.clone());
                                hit = true;
                            }
                        }
                    }
                    if hit {
                        w.changed.insert(id);
                    }
                }
                _ => {}
            }
        }
        if ids {
            for id in moved {
                if let Some(to) = followed_id(&id, &old, &new) {
                    if id_problem(graph, &id, &to).is_none() && !w.nodes.contains_key(&to) {
                        w.rename(&id, &to);
                    }
                }
            }
        }
    } else {
        for id in ids_of(&w) {
            let n = w.nodes.get_mut(&id).expect("listed");
            let mut hit = false;
            for l in n.links.iter_mut().filter(|l| l.kind == old) {
                l.kind = new.clone();
                hit = true;
            }
            if n.kind.as_deref() == Some("rule") {
                if let Some(c) = n.fields.get_mut("check") {
                    hit |= rename_link_in(c, &old, &new);
                }
            }
            if hit {
                w.changed.insert(id);
            }
        }
    }
    Ok(w.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load;

    fn graph(files: &[(&str, &str)]) -> Graph {
        load(files.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect()).0
    }

    const MODEL: &[(&str, &str)] = &[
        ("type-table.md", "---\nid: type-table\ntype: type\nfields: {name: table}\n---\n"),
        (
            "link-type-contains.md",
            "---\nid: link-type-contains\ntype: link-type\nfields: {name: contains, from: [table], to: [column]}\n---\n",
        ),
        ("table-a.md", "---\nid: table-a\ntype: table\nlinks: [{type: contains, to: column-x}]\n---\n"),
        ("column-x.md", "---\nid: column-x\ntype: column\nparent: table-a\n---\n"),
        (
            "r.md",
            "---\nid: r\ntype: rule\nfields: {applies_to: {type: table, under: table-a}, check: {require_link: [{type: contains, node_type: column}], acyclic: contains}}\n---\n",
        ),
        ("p.md", "---\nid: p\ntype: plane\n---\n"),
        ("m.md", "---\nid: m\ntype: component\nplane: p\n---\n"),
    ];

    #[test]
    fn renames_a_node_everywhere_it_is_named() {
        let g = graph(MODEL);
        let e = rename_node(&g, "table-a", "tbl-a").unwrap();
        assert_eq!(e.delete, vec!["table-a".to_string()]);
        let ids: Vec<_> = e.write.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, ["column-x", "r", "tbl-a"]);
        let col = e.write.iter().find(|n| n.id == "column-x").unwrap();
        assert_eq!(col.parent.as_deref(), Some("tbl-a"));
        let r = e.write.iter().find(|n| n.id == "r").unwrap();
        assert_eq!(r.fields["applies_to"]["under"], "tbl-a");
    }

    #[test]
    fn renames_a_plane_and_its_members() {
        let e = rename_node(&graph(MODEL), "p", "layer").unwrap();
        assert_eq!(e.write.iter().find(|n| n.id == "m").unwrap().plane.as_deref(), Some("layer"));
    }

    #[test]
    fn refuses_a_bad_or_taken_id() {
        let g = graph(MODEL);
        assert!(rename_node(&g, "table-a", "column-x").unwrap_err().contains("already exists"));
        assert!(rename_node(&g, "table-a", "Column-X").unwrap_err().contains("capitals"));
        assert!(rename_node(&g, "table-a", "a/b").is_err());
        assert!(rename_node(&g, "nope", "x").unwrap_err().contains("does not exist"));
    }

    #[test]
    fn renames_a_type_and_its_users() {
        let e = rename_def(&graph(MODEL), "type-table", "data-table", true).unwrap();
        let by = |id: &str| e.write.iter().find(|n| n.id == id).unwrap_or_else(|| panic!("{id} not written"));
        assert_eq!(by("type-table").fields["name"], "data-table");
        assert_eq!(by("data-table-a").kind.as_deref(), Some("data-table"));
        assert_eq!(by("column-x").parent.as_deref(), Some("data-table-a"));
        assert_eq!(by("r").fields["applies_to"]["type"], "data-table");
        assert_eq!(by("link-type-contains").fields["from"][0], "data-table");
        assert!(e.delete.contains(&"table-a".to_string()));
    }

    #[test]
    fn renames_a_type_inside_an_any_of_list() {
        let mut files: Vec<(&str, &str)> = MODEL.to_vec();
        files.push(("r-any.md", "---\nid: r-any\ntype: rule\nfields: {applies_to: {type: [table, other]}, check: {require_fields: [x]}}\n---\n"));
        let e = rename_def(&graph(&files), "type-table", "data-table", true).unwrap();
        let r = e.write.iter().find(|n| n.id == "r-any").expect("rule written");
        assert_eq!(r.fields["applies_to"]["type"], serde_json::json!(["data-table", "other"]));
    }

    #[test]
    fn without_ids_the_ids_stay() {
        let e = rename_def(&graph(MODEL), "type-table", "data-table", false).unwrap();
        assert!(e.delete.is_empty());
        assert_eq!(e.write.iter().find(|n| n.id == "table-a").unwrap().kind.as_deref(), Some("data-table"));
    }

    #[test]
    fn renames_a_link_type_in_links_and_checks() {
        let e = rename_def(&graph(MODEL), "link-type-contains", "holds", false).unwrap();
        let by = |id: &str| e.write.iter().find(|n| n.id == id).unwrap();
        assert_eq!(by("table-a").links[0].kind, "holds");
        assert_eq!(by("r").fields["check"]["require_link"][0]["type"], "holds");
        assert_eq!(by("r").fields["check"]["acyclic"], "holds");
    }

    #[test]
    fn refuses_a_name_that_is_taken_and_does_nothing_for_the_same_name() {
        let g = graph(MODEL);
        assert!(rename_def(&g, "type-table", "table", false).unwrap().write.is_empty());
        let mut files: Vec<(&str, &str)> = MODEL.to_vec();
        files.push(("type-column.md", "---\nid: type-column\ntype: type\nfields: {name: column}\n---\n"));
        assert!(rename_def(&graph(&files), "type-table", "column", false).unwrap_err().contains("already a type"));
        assert!(rename_def(&g, "table-a", "x", false).unwrap_err().contains("not a type"));
    }
}
