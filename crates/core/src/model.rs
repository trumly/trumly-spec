//! The in-memory shape of a project: nodes, links, and the graph that holds them.
//! Mirrors spec/FORMAT.md section 2. Nothing here reads files; that is the parser's job.

use std::collections::BTreeMap;

// serde_json::Value is "any JSON value": null, bool, number, string, list or map.
// Node fields are free-form, so we need a type that can hold any of those.
use serde_json::Value;

/// Free-form key/value data. BTreeMap (not HashMap) keeps keys sorted,
/// which makes our output deterministic, and the fixtures compare output exactly.
pub type Fields = BTreeMap<String, Value>;

/// A typed, directed edge. It is stored on the node it starts from; the target is only an id.
/// Holding an id (a String) instead of a reference to the target Node avoids
/// self-referential structs, which Rust's ownership rules make painful.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// The link type. The spec calls this `type`, which is a reserved word in Rust.
    pub kind: String, // the spec calls this `type`; that is a reserved word in Rust
    /// Id of the target node.
    pub to: String, // id of the target node
    /// An optional key the file gave this link.
    pub key: Option<String>, // Option = "may be absent"; Rust has no null
    /// Extra data carried on the link.
    pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq)]
/// One file of the project: a node with its front matter, links and Markdown body.
pub struct Node {
    /// The node's id; it matches the file name without `.md`.
    pub id: String,
    /// The node's `type`. `None` if the file had none, which is a structure finding, not a load error.
    pub kind: Option<String>, // None means the file had no `type`; that is a finding, not a load error
    /// Display title; the id when the file gives none.
    pub title: String, // defaults to the id when the file has none
    /// Id of the parent node, if any.
    pub parent: Option<String>,
    /// Id of the plane the node sits in, if any.
    pub plane: Option<String>,
    /// The other front matter fields.
    pub fields: Fields,
    /// Links from this node to other nodes.
    pub links: Vec<Link>,
    /// The Markdown body.
    pub body: String,
}

/// The whole project: every node, looked up by id. BTreeMap again, so iteration is in id order,
/// which is exactly the order the spec requires for output.
#[derive(Debug, Default)]
pub struct Graph {
    nodes: BTreeMap<String, Node>,
}

impl Graph {
    /// An empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node. If the id is already taken the existing node is kept ("first wins", the
    /// spec's duplicate rule) and the rejected node is handed back in `Err`.
    pub fn insert(&mut self, node: Node) -> Result<(), Node> {
        if self.nodes.contains_key(&node.id) {
            return Err(node);
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Look a node up. Returns a borrowed reference, not a copy: `Option<&Node>`.
    pub fn get(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// The scratch plane a node sits in, if it is in one: its `plane` names a `plane` node with `fields.scratch: true`.
    /// A scratch is a sandbox: rules skip its nodes and nothing outside it may link to or parent them.
    pub fn scratch_of<'a>(&'a self, node: &'a Node) -> Option<&'a str> {
        let p = node.plane.as_deref()?;
        let plane = self.get(p)?;
        (plane.kind.as_deref() == Some("plane") && plane.fields.get("scratch") == Some(&serde_json::Value::Bool(true)))
            .then_some(p)
    }

    /// The number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True when the graph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Every node in id order. `impl Iterator` means "something you can loop over".
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            kind: Some("thing".to_string()),
            title: id.to_string(),
            parent: parent.map(|p| p.to_string()),
            plane: None,
            fields: Fields::new(),
            links: vec![],
            body: String::new(),
        }
    }

    #[test]
    fn insert_and_get() {
        let mut g = Graph::new();
        assert!(g.insert(node("a", None)).is_ok());
        assert_eq!(g.len(), 1);
        assert_eq!(g.get("a").unwrap().title, "a");
        assert!(g.get("missing").is_none());
    }

    #[test]
    fn first_duplicate_wins() {
        let mut g = Graph::new();
        let mut first = node("a", None);
        first.title = "first".into();
        let mut second = node("a", None);
        second.title = "second".into();
        assert!(g.insert(first).is_ok());
        assert!(g.insert(second).is_err());
        assert_eq!(g.get("a").unwrap().title, "first");
    }

    #[test]
    fn iterates_in_id_order() {
        let mut g = Graph::new();
        for id in ["c", "a", "b"] {
            g.insert(node(id, None)).unwrap();
        }
        let ids: Vec<&str> = g.nodes().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn links_are_stored_on_the_source() {
        let mut n = node("a", None);
        n.links.push(Link { kind: "uses".into(), to: "b".into(), key: None, fields: Fields::new() });
        assert_eq!(n.links.len(), 1);
        assert_eq!(n.links[0].to, "b");
    }
}
