//! Turns the text of one node file into a `Node`. Pure: takes a string, returns a value.
//! Mirrors `parse_node` in reference/core.py and section 1 to 3 of spec/FORMAT.md.

use saphyr::{LoadableYamlNode, ScalarOwned, YamlOwned};
use serde_json::{Map, Number, Value};

use crate::model::{Fields, Link, Node};

/// Why a file could not be loaded. The spec names two of the four load errors here;
/// the other two (filename mismatch, duplicate id) need the filename and the rest of the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The front matter is missing or is not valid YAML.
    InvalidFrontmatter,
    /// The node has no `id`.
    MissingId,
}

impl ParseError {
    /// The name used in `check --json`.
    pub fn as_str(self) -> &'static str {
        match self {
            ParseError::InvalidFrontmatter => "invalid_frontmatter",
            ParseError::MissingId => "missing_id",
        }
    }
}

/// Split `---\n<yaml>\n---\n<body>` into (yaml, body).
/// Returns None if the text does not open with a `---` line or never closes it.
fn split_frontmatter(text: &str) -> Option<(String, String)> {
    // Windows line endings become plain ones, so the rest of the code sees one kind.
    let text = text.replace("\r\n", "\n");
    // `lines` keeps no terminators, so we rebuild with `\n` where we cut.
    let mut lines = text.split('\n');
    if lines.next()?.trim_end_matches([' ', '\t']) != "---" {
        return None;
    }
    let mut yaml: Vec<&str> = Vec::new();
    for line in lines.by_ref() {
        if line.trim_end_matches([' ', '\t']) == "---" {
            // Everything left over is the body.
            let body: Vec<&str> = lines.collect();
            return Some((yaml.join("\n"), body.join("\n").trim().to_string()));
        }
        yaml.push(line);
    }
    None
}

/// YAML value -> JSON value, because node fields are stored as JSON values.
/// Returns None for things JSON cannot hold (aliases, tags, non-string map keys, NaN).
fn to_json(y: &YamlOwned) -> Option<Value> {
    match y {
        YamlOwned::Value(s) => match s {
            ScalarOwned::Null => Some(Value::Null),
            ScalarOwned::Boolean(b) => Some(Value::Bool(*b)),
            ScalarOwned::Integer(i) => Some(Value::from(*i)),
            ScalarOwned::FloatingPoint(f) => Number::from_f64(f.into_inner()).map(Value::Number),
            ScalarOwned::String(s) => Some(Value::String(s.clone())),
        },
        YamlOwned::Sequence(seq) => seq.iter().map(to_json).collect::<Option<Vec<_>>>().map(Value::Array),
        YamlOwned::Mapping(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let key = k.as_str()?; // only string keys
                out.insert(key.to_string(), to_json(v)?);
            }
            Some(Value::Object(out))
        }
        _ => None,
    }
}

/// A string that must be present and non-empty (ids, link types, link targets).
fn non_empty(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// An optional string key: absent or null is fine, anything else must be a string.
/// Outer Option = valid?, inner Option = present?
fn opt_string(m: &Map<String, Value>, key: &str) -> Option<Option<String>> {
    match m.get(key) {
        None | Some(Value::Null) => Some(None),
        Some(Value::String(s)) => Some(Some(s.clone())),
        _ => None,
    }
}

/// An optional map: absent or null becomes empty.
fn opt_map(m: &Map<String, Value>, key: &str) -> Option<Fields> {
    match m.get(key) {
        None | Some(Value::Null) => Some(Fields::new()),
        Some(Value::Object(o)) => Some(o.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        _ => None,
    }
}

fn parse_link(v: &Value) -> Option<Link> {
    let m = v.as_object()?;
    let kind = non_empty(m.get("type")?)?;
    let to = non_empty(m.get("to")?)?;
    // An empty key means "no key", as in the reference.
    let key = opt_string(m, "key")?.filter(|k| !k.is_empty());
    let fields = opt_map(m, "fields")?;
    Some(Link { kind, to, key, fields })
}

/// Build a node from already-parsed frontmatter (a JSON map) and a body. Shared by the file parser
/// and by anything that receives a node as JSON (the server, the editor).
pub fn node_from_map(fm: &Map<String, Value>, body: String) -> Result<Node, ParseError> {
    use ParseError::*;

    // `id` first: missing (or null) is its own error, a wrong type is a generic one.
    let id = match fm.get("id") {
        None | Some(Value::Null) => return Err(MissingId),
        Some(v) => non_empty(v).ok_or(InvalidFrontmatter)?,
    };

    let kind = opt_string(fm, "type").ok_or(InvalidFrontmatter)?;
    let title = opt_string(fm, "title").ok_or(InvalidFrontmatter)?;
    let parent = opt_string(fm, "parent").ok_or(InvalidFrontmatter)?;
    let plane = opt_string(fm, "plane").ok_or(InvalidFrontmatter)?;
    let fields = opt_map(fm, "fields").ok_or(InvalidFrontmatter)?;

    let links = match fm.get("links") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) => a.iter().map(parse_link).collect::<Option<Vec<_>>>().ok_or(InvalidFrontmatter)?,
        _ => return Err(InvalidFrontmatter),
    };

    Ok(Node {
        // A missing or empty title falls back to the id.
        title: title.filter(|t| !t.is_empty()).unwrap_or_else(|| id.clone()),
        id,
        kind,
        parent,
        plane,
        fields,
        links,
        body,
    })
}

/// Parse the text of one node file.
pub fn parse_node(text: &str) -> Result<Node, ParseError> {
    use ParseError::InvalidFrontmatter;

    let (yaml_text, body) = split_frontmatter(text).ok_or(InvalidFrontmatter)?;
    let docs = YamlOwned::load_from_str(&yaml_text).map_err(|_| InvalidFrontmatter)?;
    let doc = docs.first().ok_or(InvalidFrontmatter)?;
    match to_json(doc) {
        Some(Value::Object(m)) => node_from_map(&m, body),
        _ => Err(InvalidFrontmatter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal() {
        let n = parse_node("---\nid: a\ntype: thing\n---\n").unwrap();
        assert_eq!(n.id, "a");
        assert_eq!(n.kind.as_deref(), Some("thing"));
        assert_eq!(n.title, "a"); // defaults to id
        assert_eq!(n.body, "");
    }

    #[test]
    fn full_node() {
        let src = "---\nid: a\ntype: t\ntitle: Alpha\nparent: g\nplane: p\nfields:\n  n: 1\n  ok: true\nlinks:\n  - type: uses\n    to: b\n    key: k\n    fields: { verbs: [append] }\n---\n\n# Hi\n";
        let n = parse_node(src).unwrap();
        assert_eq!(n.title, "Alpha");
        assert_eq!(n.parent.as_deref(), Some("g"));
        assert_eq!(n.fields["n"], Value::from(1));
        assert_eq!(n.fields["ok"], Value::Bool(true));
        assert_eq!(n.links[0].key.as_deref(), Some("k"));
        assert_eq!(n.links[0].fields["verbs"], serde_json::json!(["append"]));
        assert_eq!(n.body, "# Hi");
    }

    #[test]
    fn crlf_and_dashes_in_body() {
        let n = parse_node("---\r\nid: a\r\ntype: doc\r\n---\r\n\r\nText\r\n---\r\nmore\r\n").unwrap();
        assert_eq!(n.body, "Text\n---\nmore");
    }

    #[test]
    fn yaml_12_keeps_no_as_a_string() {
        let n = parse_node("---\nid: a\ntype: t\nfields:\n  x: no\n---\n").unwrap();
        assert_eq!(n.fields["x"], Value::String("no".into()));
    }

    #[test]
    fn errors() {
        assert_eq!(parse_node("no frontmatter"), Err(ParseError::InvalidFrontmatter));
        assert_eq!(parse_node("---\nid: [unclosed\n---\n"), Err(ParseError::InvalidFrontmatter));
        assert_eq!(parse_node("---\n- a\n- b\n---\n"), Err(ParseError::InvalidFrontmatter));
        assert_eq!(parse_node("---\ntype: t\n---\n"), Err(ParseError::MissingId));
        assert_eq!(parse_node("---\nid: a\nparent: 5\n---\n"), Err(ParseError::InvalidFrontmatter));
        assert_eq!(parse_node("---\nid: a\nlinks:\n  - type: x\n---\n"), Err(ParseError::InvalidFrontmatter));
    }
}
