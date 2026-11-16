//! Writes a `Node` back out as a Markdown file. The one place that decides what a canonical node file
//! looks like, so files written by the editor, the CLI and the server are all byte-identical.
//!
//! Layout: id, type, title (only if it differs from the id), parent, plane, fields, links, then the body.

use std::borrow::Cow;

use saphyr::{Mapping, Scalar, Yaml, YamlEmitter};
use serde_json::Value;

use crate::model::{Fields, Node};

type Y = Yaml<'static>;

fn string(s: &str) -> Y {
    Yaml::Value(Scalar::String(Cow::Owned(s.to_string())))
}

/// JSON value -> YAML value (the reverse of the conversion in parse.rs).
fn to_yaml(v: &Value) -> Y {
    match v {
        Value::Null => Yaml::Value(Scalar::Null),
        Value::Bool(b) => Yaml::Value(Scalar::Boolean(*b)),
        Value::Number(n) => match n.as_i64() {
            Some(i) => Yaml::Value(Scalar::Integer(i)),
            None => Yaml::Value(Scalar::FloatingPoint(n.as_f64().unwrap_or(0.0).into())),
        },
        Value::String(s) => string(s),
        Value::Array(a) => Yaml::Sequence(a.iter().map(to_yaml).collect()),
        Value::Object(o) => {
            let mut m = Mapping::new();
            for (k, v) in o {
                m.insert(string(k), to_yaml(v));
            }
            Yaml::Mapping(m)
        }
    }
}

fn fields_to_yaml(f: &Fields) -> Y {
    let mut m = Mapping::new();
    for (k, v) in f {
        m.insert(string(k), to_yaml(v));
    }
    Yaml::Mapping(m)
}

/// The file text for a node: `---`, YAML frontmatter, `---`, then the Markdown body.
pub fn node_to_markdown(n: &Node) -> String {
    let mut m = Mapping::new();
    m.insert(string("id"), string(&n.id));
    if let Some(k) = n.kind.as_deref().filter(|k| !k.is_empty()) {
        m.insert(string("type"), string(k));
    }
    if n.title != n.id {
        m.insert(string("title"), string(&n.title));
    }
    if let Some(p) = &n.parent {
        m.insert(string("parent"), string(p));
    }
    if let Some(p) = &n.plane {
        m.insert(string("plane"), string(p));
    }
    if !n.fields.is_empty() {
        m.insert(string("fields"), fields_to_yaml(&n.fields));
    }
    if !n.links.is_empty() {
        let links = n
            .links
            .iter()
            .map(|l| {
                let mut lm = Mapping::new();
                lm.insert(string("type"), string(&l.kind));
                lm.insert(string("to"), string(&l.to));
                if let Some(k) = &l.key {
                    lm.insert(string("key"), string(k));
                }
                if !l.fields.is_empty() {
                    lm.insert(string("fields"), fields_to_yaml(&l.fields));
                }
                Yaml::Mapping(lm)
            })
            .collect();
        m.insert(string("links"), Yaml::Sequence(links));
    }

    let mut out = String::new();
    let mut emitter = YamlEmitter::new(&mut out);
    emitter.multiline_strings(true);
    emitter.dump(&Yaml::Mapping(m)).expect("writing to a String cannot fail");

    // The emitter starts with its own `---` and does not end with a newline.
    out.push_str("\n---\n");
    let body = n.body.trim();
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Link;
    use crate::parse::parse_node;
    use crate::rules::json_eq;
    use serde_json::json;

    fn node(fields: Value) -> Node {
        Node {
            id: "a".into(),
            kind: Some("thing".into()),
            title: "a".into(),
            parent: None,
            plane: None,
            fields: fields.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            links: vec![],
            body: String::new(),
        }
    }

    fn same(a: &Node, b: &Node) -> bool {
        a.id == b.id
            && a.kind == b.kind
            && a.title == b.title
            && a.parent == b.parent
            && a.plane == b.plane
            && a.body == b.body
            && json_eq(&json!(a.fields), &json!(b.fields))
            && a.links.len() == b.links.len()
            && a.links.iter().zip(&b.links).all(|(x, y)| {
                x.kind == y.kind && x.to == y.to && x.key == y.key && json_eq(&json!(x.fields), &json!(y.fields))
            })
    }

    #[test]
    fn layout_is_canonical() {
        let mut n = node(json!({"note": "hi"}));
        n.title = "Alpha".into();
        n.parent = Some("g".into());
        n.links.push(Link { kind: "uses".into(), to: "b".into(), key: Some("k".into()), fields: Fields::new() });
        n.body = "# Heading\n\nText.".into();
        let text = node_to_markdown(&n);
        assert_eq!(
            text,
            "---\nid: a\ntype: thing\ntitle: Alpha\nparent: g\nfields:\n  note: hi\nlinks:\n  - type: uses\n    to: b\n    key: k\n---\n\n# Heading\n\nText.\n"
        );
    }

    #[test]
    fn round_trips_awkward_values() {
        let awkward = json!({
            "yes": "no", "num_string": "123", "bool_string": "true", "null_string": "null", "empty": "",
            "colon": "a: b", "hash": "# not a comment", "dash": "- item", "lead": "  spaced", "trail": "spaced  ",
            "multi": "line one\nline two\n", "multi2": "no trailing newline\nsecond", "quote": "it's \"quoted\"",
            "unicode": "naïve — café ✓", "date": "2026-10-19", "float_string": "1.0", "tilde": "~", "brace": "{x}", "bracket": "[x]",
            "int": 42, "neg": -7, "float": 1.5, "t": true, "f": false, "nil": null,
            "list": [1, "two", [3], {"k": "v"}], "map": {"nested": {"deep": ["x"]}}, "emptylist": [], "emptymap": {}
        });
        let n = node(awkward);
        let text = node_to_markdown(&n);
        let back = parse_node(&text).unwrap_or_else(|e| panic!("{e:?}\n{text}"));
        assert!(same(&n, &back), "round trip differs:\n{text}\n{:?}\n{:?}", n.fields, back.fields);
    }

    #[test]
    fn formatting_twice_changes_nothing() {
        let n = node(json!({"multi": "a\nb\n", "list": [1, 2]}));
        let once = node_to_markdown(&n);
        let twice = node_to_markdown(&parse_node(&once).unwrap());
        assert_eq!(once, twice);
    }
}
