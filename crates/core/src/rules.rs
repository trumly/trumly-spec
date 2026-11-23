//! Runs the rules: which rule applies to which node, and does the node pass?
//! Mirrors `matches`, `run_check` and `evaluate` in reference/core.py and section 5 of the spec.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::{Graph, Link, Node};
use crate::names::{Names, Scope, def_ns};

/// Node types that rules never apply to.
pub(crate) const RESERVED: [&str; 7] = ["plane", "rule", "group", "type", "link-type", "pack", "derived"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The outcome of one rule on one node.
pub enum Status {
    /// The rule holds.
    Pass,
    /// The rule is broken.
    Fail,
    /// The rule is broken, but its severity only advises.
    Advisory,
    /// The rule's check uses something this implementation does not understand. Never a silent pass.
    Unsupported,
}

impl Status {
    /// The name used in `check --json`.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Advisory => "advisory",
            Status::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// One rule applied to one node.
pub struct RuleResult {
    /// Id of the node that was checked.
    pub node: String,
    /// Id of the rule.
    pub rule: String,
    /// The rule's severity, as the rule states it.
    pub severity: String,
    /// The outcome.
    pub status: Status,
}

/// A node that a failed `require_link` says should exist: "a link of this type to something of
/// this type, in this plane". The editor draws these as ghosts.
#[derive(Debug, Clone, PartialEq)]
pub struct Ghost {
    /// Id of the node the missing link starts from.
    pub node: String,
    /// Id of the rule that asks for it.
    pub rule: String,
    /// The wanted link type.
    pub link: Value, // the wanted link type (Null if the rule gave none)
    /// The wanted node type.
    pub to_type: Value, // Null if unspecified
    /// The plane the missing node belongs in.
    pub plane: Value, // `to_plane` if given, otherwise the node's own plane
}

#[derive(Debug, Default)]
/// Everything the rules found in a graph.
pub struct Evaluation {
    /// One result per rule and node it applies to.
    pub results: Vec<RuleResult>,
    /// Nodes that failed `require_link` rules say should exist.
    pub ghosts: Vec<Ghost>,
}

/// JSON equality, the way the spec defines it: `true` is never `1`, but `1` equals `1.0`,
/// and this holds inside lists and maps. serde_json's own `==` would say `1 != 1.0`.
pub fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let int = |n: &serde_json::Number| n.is_i64() || n.is_u64();
            if int(x) && int(y) {
                x == y // two integers: compare exactly, no float rounding
            } else {
                x.as_f64() == y.as_f64()
            }
        }
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| json_eq(p, q)),
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w)))
        }
        // Everything else (null, bool, string, or mismatched kinds) is plain equality.
        _ => a == b,
    }
}

/// "Truthy" in the reference's sense, used for `to_plane or node.plane`.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn opt_value(s: &Option<String>) -> Value {
    s.as_ref().map_or(Value::Null, |s| Value::String(s.clone()))
}

/// The node itself, then its parents. Stops at a missing parent or a repeat.
fn ancestors<'a>(graph: &'a Graph, id: &'a str) -> Vec<&'a str> {
    let mut chain: Vec<&str> = Vec::new();
    let mut cur = Some(id);
    while let Some(c) = cur {
        let Some(node) = graph.get(c) else { break };
        if chain.contains(&c) {
            break;
        }
        chain.push(c);
        cur = node.parent.as_deref();
    }
    chain
}

/// Does `node` match an `applies_to` filter? Every key must match.
pub(crate) fn matches(graph: &Scope, node: &Node, filter: &serde_json::Map<String, Value>) -> bool {
    // Keys all have to match. A list under a key matches when any one of its items does ("type: [a, b]" is a or b).
    filter.iter().all(|(key, want)| {
        let wants = items(want);
        match key.as_str() {
            "type" => wants.iter().any(|w| graph.same_type(&opt_value(&node.kind), w)),
            "plane" => wants.iter().any(|w| json_eq(&opt_value(&node.plane), w)),
            // Strict ancestor: the node itself does not count (`skip(1)`).
            "under" => {
                let up = ancestors(graph, &node.id);
                wants.iter().any(|w| w.as_str().is_some_and(|w| up.iter().skip(1).any(|a| *a == w)))
            }
            // One id, or a list of ids; anything that is not a string never matches.
            "id" => wants.iter().any(|x| x.as_str() == Some(node.id.as_str())),
            // Any other key is a field. A missing field never matches, even against null.
            // A field path matches when any place it reaches holds any of the values (`methods[].verb: POST`: some method is a POST).
            _ => resolve(|k| node.fields.get(k), key).is_some_and(|slots| {
                slots.iter().any(|s| s.is_some_and(|have| wants.iter().any(|w| json_eq(have, w))))
            }),
        }
    })
}

// ---------- checks ----------
// Each check returns Pass, Fail, or Bad (the check is malformed or not understood). Bad inside one check
// always wins over Fail inside that same check, exactly as in the reference.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pass,
    Fail,
    Bad,
}

type Obj = serde_json::Map<String, Value>;

/// A check value may be one item or a list of items.
fn items(v: &Value) -> Vec<&Value> {
    match v {
        Value::Array(a) => a.iter().collect(),
        other => vec![other],
    }
}

/// Present means: there, and neither null nor the empty string.
fn present(v: Option<&Value>) -> bool {
    v.is_some_and(|v| !v.is_null() && v != &Value::String(String::new()))
}

fn is_number(v: &Value) -> bool {
    v.is_number()
}

fn cmp_numbers(a: &serde_json::Number, b: &serde_json::Number) -> Option<std::cmp::Ordering> {
    if let (Some(x), Some(y)) = (a.as_i64(), b.as_i64()) {
        return Some(x.cmp(&y)); // two integers: exact
    }
    a.as_f64()?.partial_cmp(&b.as_f64()?)
}

fn non_neg_int(v: &Value) -> Option<u64> {
    v.as_u64() // floats such as 1.0 are not integers here
}

fn type_is(v: &Value, name: &str) -> Option<bool> {
    Some(match name {
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.is_i64() || v.is_u64(),
        "boolean" => v.is_boolean(),
        "list" => v.is_array(),
        "map" => v.is_object(),
        "null" => v.is_null(),
        _ => return None,
    })
}

/// One resolved place a field path points at. `None` means the path ran into something missing.
type Slot<'a> = Option<&'a Value>;

/// Resolves a field path against a node's fields. A name that is itself a top-level field is used as is.
/// Otherwise a path is names joined by `.`, and a name followed by `[]` means "each item of that list":
/// `endpoint.params`, `methods[].verb`, `tags[]`. Returns one slot per place reached: an empty list reaches
/// none, a missing step or a non-list under `[]` reaches one missing slot. `None` means the path is malformed.
pub(crate) fn resolve<'a>(root: impl Fn(&str) -> Option<&'a Value>, path: &str) -> Option<Vec<Slot<'a>>> {
    if let Some(v) = root(path) {
        return Some(vec![Some(v)]);
    }
    if !path.contains('.') && !path.contains('[') && !path.contains(']') {
        return Some(vec![None]);
    }
    let mut slots: Vec<Slot> = Vec::new();
    for (i, seg) in path.split('.').enumerate() {
        let (name, each) = match seg.strip_suffix("[]") {
            Some(n) => (n, true),
            None => (seg, false),
        };
        if name.is_empty() || name.contains('[') || name.contains(']') {
            return None;
        }
        let mut next: Vec<Slot> = Vec::new();
        if i == 0 {
            next.push(root(name));
        } else {
            for s in &slots {
                next.push(s.and_then(Value::as_object).and_then(|o| o.get(name)));
            }
        }
        if each {
            let mut items: Vec<Slot> = Vec::new();
            for s in next {
                match s {
                    Some(Value::Array(a)) => items.extend(a.iter().map(Some)),
                    _ => items.push(None),
                }
            }
            next = items;
        }
        slots = next;
    }
    Some(slots)
}

fn check_require_fields(node: &Node, names: &Value) -> Outcome {
    let Some(names) = names.as_array() else { return Outcome::Bad };
    let mut all = true;
    for n in names {
        let Some(n) = n.as_str() else { return Outcome::Bad };
        let Some(slots) = resolve(|k| node.fields.get(k), n) else { return Outcome::Bad };
        all &= slots.iter().all(|s| present(*s));
    }
    if all { Outcome::Pass } else { Outcome::Fail }
}

const FIELD_IS_KEYS: [&str; 11] = [
    "in",
    "min",
    "max",
    "type",
    "optional",
    "min_items",
    "max_items",
    "unique",
    "min_length",
    "max_length",
    "max_bytes",
];

/// One node's calculated values for rules, by name: one value is text, several are a list of text. A name that could not be worked
/// out is left out, so a rule sees it as an absent field.
pub type Calc = BTreeMap<String, Value>;

/// `unique` is `true` (the items differ from each other) or a field path inside each item (that field differs).
fn unique_ok(list: &[Value], by: &Value) -> bool {
    let key = |item: &Value| -> Option<Value> {
        match by.as_str() {
            Some(p) => item
                .as_object()
                .and_then(|o| resolve(|k| o.get(k), p))
                .and_then(|s| s.first().copied().flatten().cloned()),
            None => Some(item.clone()),
        }
    };
    let keys: Vec<Value> = list.iter().filter_map(key).collect(); // items without the key are not compared
    keys.iter().enumerate().all(|(i, k)| !keys[..i].iter().any(|j| json_eq(j, k)))
}

/// A name the node has no field of is read from its own `id` and `title`, then from `calc` (its calculated values), so a rule can check
/// a node's name or a derived value like a field.
fn check_field_is(node: &Node, calc: Option<&Calc>, specs: &Value) -> Outcome {
    let Some(specs) = specs.as_object() else { return Outcome::Bad };
    let mut outcome = Outcome::Pass;
    // the node's own `id` and `title` can be checked like fields (a node's title is its id when the file gives none)
    let own: Calc = [("id", node.id.clone()), ("title", node.title.clone())]
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::String(v)))
        .collect();
    for (name, spec) in specs {
        let Some(spec) = spec.as_object() else { return Outcome::Bad };
        if spec.keys().any(|k| !FIELD_IS_KEYS.contains(&k.as_str())) {
            return Outcome::Bad;
        }
        if spec.get("in").is_some_and(|v| !v.is_array()) {
            return Outcome::Bad;
        }
        if ["min", "max"].iter().any(|k| spec.get(*k).is_some_and(|v| !is_number(v))) {
            return Outcome::Bad;
        }
        if ["min_items", "max_items", "min_length", "max_length", "max_bytes"]
            .iter()
            .any(|k| spec.get(*k).is_some_and(|v| non_neg_int(v).is_none()))
        {
            return Outcome::Bad;
        }
        match spec.get("unique") {
            None | Some(Value::Bool(true)) => {}
            Some(Value::String(p)) if !p.is_empty() && !p.contains("[]") && resolve(|_| None, p).is_some() => {}
            _ => return Outcome::Bad,
        }
        if spec.get("type").is_some_and(|t| t.as_str().and_then(|t| type_is(&Value::Null, t)).is_none()) {
            return Outcome::Bad;
        }
        // a calculated name that gives several values is checked value by value, like a list written `name[]`
        let calculated = calc
            .and_then(|c| c.get(name.as_str()))
            .filter(|_| !node.fields.contains_key(name.as_str()) && !own.contains_key(name.as_str()));
        let slots: Vec<Slot> = match calculated {
            Some(Value::Array(many)) => many.iter().map(Some).collect(),
            _ => match resolve(
                |k| node.fields.get(k).or_else(|| own.get(k)).or_else(|| calc.and_then(|c| c.get(k))),
                name,
            ) {
                Some(s) => s,
                None => return Outcome::Bad,
            },
        };
        for slot in slots {
            let Some(v) = slot else {
                if spec.get("optional") != Some(&Value::Bool(true)) {
                    outcome = Outcome::Fail;
                }
                continue;
            };
            let num = v.as_number();
            let bound = |k: &str, want: std::cmp::Ordering| -> bool {
                // `min` needs v >= min, `max` needs v <= max
                match (num, spec[k].as_number()) {
                    (Some(x), Some(y)) => {
                        cmp_numbers(x, y).is_some_and(|o| o == want || o == std::cmp::Ordering::Equal)
                    }
                    _ => false,
                }
            };
            if spec.get("in").and_then(Value::as_array).is_some_and(|list| !list.iter().any(|x| json_eq(v, x))) {
                outcome = Outcome::Fail;
            }
            if spec.contains_key("min") && !bound("min", std::cmp::Ordering::Greater) {
                outcome = Outcome::Fail;
            }
            if spec.contains_key("max") && !bound("max", std::cmp::Ordering::Less) {
                outcome = Outcome::Fail;
            }
            if let Some(t) = spec.get("type").and_then(Value::as_str) {
                if type_is(v, t) != Some(true) {
                    outcome = Outcome::Fail;
                }
            }
            let len = v.as_array().map(|a| a.len() as u64);
            if spec.get("min_items").and_then(non_neg_int).is_some_and(|m| len.is_none_or(|n| n < m)) {
                outcome = Outcome::Fail;
            }
            if spec.get("max_items").and_then(non_neg_int).is_some_and(|m| len.is_none_or(|n| n > m)) {
                outcome = Outcome::Fail;
            }
            if let Some(by) = spec.get("unique") {
                if !v.as_array().is_some_and(|a| unique_ok(a, by)) {
                    outcome = Outcome::Fail;
                }
            }
            // text lengths: characters for min_length and max_length, UTF-8 bytes for max_bytes; a value that is not text fails
            let text = v.as_str();
            if spec
                .get("min_length")
                .and_then(non_neg_int)
                .is_some_and(|m| text.is_none_or(|s| (s.chars().count() as u64) < m))
            {
                outcome = Outcome::Fail;
            }
            if spec
                .get("max_length")
                .and_then(non_neg_int)
                .is_some_and(|m| text.is_none_or(|s| (s.chars().count() as u64) > m))
            {
                outcome = Outcome::Fail;
            }
            if spec.get("max_bytes").and_then(non_neg_int).is_some_and(|m| text.is_none_or(|s| (s.len() as u64) > m)) {
                outcome = Outcome::Fail;
            }
        }
    }
    outcome
}

fn check_compare_fields(node: &Node, list: &Value) -> Outcome {
    let mut outcome = Outcome::Pass;
    for it in items(list) {
        let Some(it) = it.as_object() else { return Outcome::Bad };
        if it.keys().any(|k| !["left", "op", "right", "value"].contains(&k.as_str())) {
            return Outcome::Bad;
        }
        let (Some(left), Some(op)) = (it.get("left").and_then(Value::as_str), it.get("op").and_then(Value::as_str))
        else {
            return Outcome::Bad;
        };
        if !["==", "!=", "<", "<=", ">", ">="].contains(&op) || it.contains_key("right") == it.contains_key("value") {
            return Outcome::Bad;
        }
        let right_name = match it.get("right") {
            Some(r) => match r.as_str() {
                Some(r) => Some(r),
                None => return Outcome::Bad,
            },
            None => None,
        };
        let Some(lefts) = resolve(|k| node.fields.get(k), left) else { return Outcome::Bad };
        let r = match right_name {
            Some(name) => match resolve(|k| node.fields.get(k), name) {
                Some(s) if s.len() == 1 && !name.contains("[]") => s[0],
                _ => return Outcome::Bad, // the right side is one place; `[]` belongs on the left
            },
            None => it.get("value"),
        };
        for l in lefts {
            let (Some(l), Some(r)) = (l, r) else {
                outcome = Outcome::Fail; // a missing field never compares
                continue;
            };
            use std::cmp::Ordering::*;
            let ordering = match (l, r) {
                (Value::Number(a), Value::Number(b)) => cmp_numbers(a, b),
                (Value::String(a), Value::String(b)) => Some(a.as_str().cmp(b.as_str())),
                _ => None,
            };
            let ok = match op {
                "==" => json_eq(l, r),
                "!=" => !json_eq(l, r),
                "<" => ordering == Some(Less),
                "<=" => matches!(ordering, Some(Less | Equal)),
                ">" => ordering == Some(Greater),
                _ => matches!(ordering, Some(Greater | Equal)),
            };
            if !ok {
                outcome = Outcome::Fail;
            }
        }
    }
    outcome
}

/// For every node, the links that point at it: (the node they come from, the link).
pub(crate) type Inbound<'a> = std::collections::BTreeMap<&'a str, Vec<(&'a Node, &'a Link)>>;

pub(crate) fn build_inbound(graph: &Graph) -> Inbound<'_> {
    let mut m: Inbound = Inbound::new();
    for n in graph.nodes() {
        for l in &n.links {
            m.entry(l.to.as_str()).or_default().push((n, l));
        }
    }
    m
}

const LINK_KEYS: [&str; 13] = [
    "type",
    "dir",
    "node_type",
    "node_plane",
    "to_type",
    "to_plane",
    "node_fields",
    "same_fields",
    "ancestor_type",
    "scope",
    "with_fields",
    "min",
    "max",
];

fn non_empty_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty())
}

fn dir_of(it: &Obj) -> Option<&str> {
    match it.get("dir") {
        None => Some("out"),
        Some(Value::String(d)) if d == "out" || d == "in" => Some(d.as_str()),
        _ => None,
    }
}

fn link_item_valid(it: &Obj) -> bool {
    if it.keys().any(|k| !LINK_KEYS.contains(&k.as_str()))
        || non_empty_str(it.get("type")).is_none()
        || dir_of(it).is_none()
    {
        return false;
    }
    if it.contains_key("ancestor_type") != it.contains_key("scope") {
        return false;
    }
    if it.get("scope").is_some_and(|s| s != "same" && s != "other") {
        return false;
    }
    if it.contains_key("ancestor_type") && non_empty_str(it.get("ancestor_type")).is_none() {
        return false;
    }
    if it.get("node_fields").is_some_and(|m| !m.as_object().is_some()) {
        return false;
    }
    if let Some(w) = it.get("same_fields") {
        if !w.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
            return false;
        }
    }
    if let Some(w) = it.get("with_fields") {
        if !w.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
            return false;
        }
    }
    ["min", "max"].iter().all(|k| it.get(*k).is_none_or(|v| non_neg_int(v).is_some()))
}

fn link_bounds(it: &Obj) -> (u64, Option<u64>) {
    let max = it.get("max").and_then(non_neg_int);
    (it.get("min").and_then(non_neg_int).unwrap_or(if max.is_some() { 0 } else { 1 }), max)
}

fn nearest_ancestor<'a>(graph: &'a Scope, id: &'a str, kind: &str) -> Option<&'a str> {
    ancestors(graph, id)
        .into_iter()
        .skip(1)
        .find(|a| graph.get(a).is_some_and(|n| n.kind.as_deref().is_some_and(|k| graph.same_type_str(k, kind))))
}

/// How many links of this node (out) or to this node (in) satisfy every filter in `it`.
fn count_links(graph: &Scope, inbound: &Inbound, node: &Node, it: &Obj) -> u64 {
    let kind = it["type"].as_str().unwrap_or("");
    let pairs: Vec<(&Link, Option<&Node>)> = if dir_of(it) == Some("out") {
        node.links.iter().filter(|l| graph.same_link(&l.kind, kind)).map(|l| (l, graph.get(&l.to))).collect()
    } else {
        inbound.get(node.id.as_str()).map_or_else(Vec::new, |v| {
            v.iter().filter(|(_, l)| graph.same_link(&l.kind, kind)).map(|(src, l)| (*l, Some(*src))).collect()
        })
    };
    let want_type = it.get("node_type").or_else(|| it.get("to_type"));
    let want_plane = it.get("node_plane").or_else(|| it.get("to_plane"));
    let mut n = 0;
    for (l, other) in pairs {
        let Some(other) = other else { continue }; // a link to nothing never counts
        if want_type.is_some_and(|w| !graph.same_type(&opt_value(&other.kind), w))
            || want_plane.is_some_and(|w| !json_eq(&opt_value(&other.plane), w))
        {
            continue;
        }
        if let Some(want) = it.get("node_fields").and_then(Value::as_object) {
            if !want.iter().all(|(k, v)| other.fields.get(k).is_some_and(|have| json_eq(have, v))) {
                continue;
            }
        }
        if let Some(names) = it.get("same_fields").and_then(Value::as_array) {
            if !names.iter().filter_map(Value::as_str).all(|f| match (node.fields.get(f), other.fields.get(f)) {
                (Some(a), Some(b)) => json_eq(a, b),
                _ => false,
            }) {
                continue;
            }
        }
        if let (Some(t), Some(scope)) =
            (it.get("ancestor_type").and_then(Value::as_str), it.get("scope").and_then(Value::as_str))
        {
            let (a, b) = (nearest_ancestor(graph, &node.id, t), nearest_ancestor(graph, &other.id, t));
            let ok = match scope {
                "same" => a.is_some() && a == b,
                _ => a.is_some() && b.is_some() && a != b,
            };
            if !ok {
                continue;
            }
        }
        if let Some(names) = it.get("with_fields").and_then(Value::as_array) {
            if !names.iter().all(|f| present(f.as_str().and_then(|f| l.fields.get(f)))) {
                continue;
            }
        }
        n += 1;
    }
    n
}

fn check_require_link(graph: &Scope, inbound: &Inbound, node: &Node, list: &Value) -> Outcome {
    let mut outcome = Outcome::Pass;
    for it in items(list) {
        let Some(it) = it.as_object().filter(|it| link_item_valid(it)) else { return Outcome::Bad };
        let (lo, hi) = link_bounds(it);
        let n = count_links(graph, inbound, node, it);
        if n < lo || hi.is_some_and(|h| n > h) {
            outcome = Outcome::Fail;
        }
    }
    outcome
}

/// Sugar for `max: 0`: the same filters as `require_link`, but `min` and `max` are not accepted.
fn check_forbid_link(graph: &Scope, inbound: &Inbound, node: &Node, list: &Value) -> Outcome {
    let mut outcome = Outcome::Pass;
    for it in items(list) {
        let Some(it) =
            it.as_object().filter(|it| link_item_valid(it) && !it.contains_key("min") && !it.contains_key("max"))
        else {
            return Outcome::Bad;
        };
        if count_links(graph, inbound, node, it) > 0 {
            outcome = Outcome::Fail;
        }
    }
    outcome
}

const STEP_KEYS: [&str; 5] = ["type", "dir", "node_type", "node_plane", "node_fields"];

fn step_valid(it: &Obj) -> bool {
    !it.keys().any(|k| !STEP_KEYS.contains(&k.as_str()))
        && non_empty_str(it.get("type")).is_some()
        && dir_of(it).is_some()
        && it.get("node_fields").is_none_or(Value::is_object)
}

/// Ids at the other end of `id`'s links of this step's type that exist and match the step's filters.
fn step_targets<'a>(graph: &'a Scope, inbound: &Inbound<'a>, id: &str, step: &Obj) -> Vec<&'a str> {
    let kind = step["type"].as_str().unwrap_or("");
    let ends: Vec<&'a Node> = if dir_of(step) == Some("out") {
        graph.get(id).map_or_else(Vec::new, |n| {
            n.links.iter().filter(|l| graph.same_link(&l.kind, kind)).filter_map(|l| graph.get(&l.to)).collect()
        })
    } else {
        inbound.get(id).map_or_else(Vec::new, |v| {
            v.iter().filter(|(_, l)| graph.same_link(&l.kind, kind)).map(|(src, _)| *src).collect()
        })
    };
    ends.into_iter()
        .filter(|o| step.get("node_type").is_none_or(|w| graph.same_type(&opt_value(&o.kind), w)))
        .filter(|o| step.get("node_plane").is_none_or(|w| json_eq(&opt_value(&o.plane), w)))
        .filter(|o| {
            step.get("node_fields")
                .and_then(Value::as_object)
                .is_none_or(|want| want.iter().all(|(k, v)| o.fields.get(k).is_some_and(|h| json_eq(h, v))))
        })
        .map(|o| o.id.as_str())
        .collect()
}

/// A chain of link steps must exist from this node; each step is followed from every node the last one reached.
fn check_path(graph: &Scope, inbound: &Inbound, node: &Node, steps: &Value) -> Outcome {
    let Some(steps) = steps.as_array().filter(|a| !a.is_empty()) else { return Outcome::Bad };
    let mut parsed: Vec<&Obj> = Vec::new();
    for s in steps {
        match s.as_object().filter(|o| step_valid(o)) {
            Some(o) => parsed.push(o),
            None => return Outcome::Bad,
        }
    }
    let mut frontier: std::collections::BTreeSet<&str> = std::collections::BTreeSet::from([node.id.as_str()]);
    for step in parsed {
        frontier = frontier.iter().flat_map(|f| step_targets(graph, inbound, f, step)).collect();
        if frontier.is_empty() {
            return Outcome::Fail;
        }
    }
    Outcome::Pass
}

/// Fails if this node can reach itself by following links of the given type(s).
fn check_acyclic(graph: &Scope, node: &Node, types: &Value) -> Outcome {
    let list = items(types);
    let kinds: Vec<&str> = list.iter().filter_map(|v| non_empty_str(Some(*v))).collect();
    if list.is_empty() || kinds.len() != list.len() {
        return Outcome::Bad;
    }
    for kind in kinds {
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        let mut stack: Vec<&str> =
            node.links.iter().filter(|l| graph.same_link(&l.kind, kind)).map(|l| l.to.as_str()).collect();
        while let Some(cur) = stack.pop() {
            if cur == node.id {
                return Outcome::Fail;
            }
            let Some(n) = graph.get(cur) else { continue };
            if seen.insert(cur) {
                stack.extend(n.links.iter().filter(|l| graph.same_link(&l.kind, kind)).map(|l| l.to.as_str()));
            }
        }
    }
    Outcome::Pass
}

fn check_require_link_fields(graph: &Scope, inbound: &Inbound, node: &Node, list: &Value) -> Outcome {
    let mut outcome = Outcome::Pass;
    for it in items(list) {
        let Some(it) = it.as_object() else { return Outcome::Bad };
        let fields_ok = it.get("fields").and_then(Value::as_array).is_some_and(|a| a.iter().all(Value::is_string));
        if it.keys().any(|k| !["type", "dir", "fields"].contains(&k.as_str()))
            || non_empty_str(it.get("type")).is_none()
            || dir_of(it).is_none()
            || !fields_ok
        {
            return Outcome::Bad;
        }
        let kind = it["type"].as_str().unwrap_or("");
        let names: Vec<&str> = it["fields"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
        let links: Vec<&Link> = if dir_of(it) == Some("out") {
            node.links.iter().filter(|l| graph.same_link(&l.kind, kind)).collect()
        } else {
            inbound.get(node.id.as_str()).map_or_else(Vec::new, |v| {
                v.iter().filter(|(_, l)| graph.same_link(&l.kind, kind)).map(|(_, l)| *l).collect()
            })
        };
        if !links.iter().all(|l| names.iter().all(|f| present(l.fields.get(*f)))) {
            outcome = Outcome::Fail;
        }
    }
    outcome
}

/// `any_of`: a non-empty list of checks (each a full check map); passes if at least one passes. If none passes and one is not
/// understood, the whole is not understood (that one might have passed). An empty list, a non-list, or an item that is not a
/// non-empty map is not understood.
fn check_any_of(graph: &Scope, inbound: &Inbound, node: &Node, calc: Option<&Calc>, val: &Value) -> Outcome {
    let Some(list) = val.as_array().filter(|a| !a.is_empty()) else { return Outcome::Bad };
    let mut maps = Vec::new();
    for it in list {
        match it.as_object().filter(|o| !o.is_empty()) {
            Some(o) => maps.push(o),
            None => return Outcome::Bad,
        }
    }
    let mut bad = false;
    for o in maps {
        match run_check(graph, inbound, node, calc, o) {
            Status::Pass => return Outcome::Pass,
            Status::Unsupported => bad = true,
            _ => {}
        }
    }
    if bad { Outcome::Bad } else { Outcome::Fail }
}

/// Pass only if every part passes and every part is understood; a failure outranks "not understood".
fn run_check(graph: &Scope, inbound: &Inbound, node: &Node, calc: Option<&Calc>, check: &Obj) -> Status {
    let mut any_fail = false;
    let mut any_bad = false;
    for (key, val) in check {
        let o = match key.as_str() {
            "require_fields" => check_require_fields(node, val),
            "field_is" => check_field_is(node, calc, val),
            "compare_fields" => check_compare_fields(node, val),
            "require_link" => check_require_link(graph, inbound, node, val),
            "forbid_link" => check_forbid_link(graph, inbound, node, val),
            "require_link_fields" => check_require_link_fields(graph, inbound, node, val),
            "path" => check_path(graph, inbound, node, val),
            "acyclic" => check_acyclic(graph, node, val),
            "any_of" => check_any_of(graph, inbound, node, calc, val),
            _ => Outcome::Bad,
        };
        any_fail |= o == Outcome::Fail;
        any_bad |= o == Outcome::Bad;
    }
    if any_fail {
        Status::Fail
    } else if any_bad {
        Status::Unsupported
    } else {
        Status::Pass
    }
}

/// One ghost per outgoing `require_link` that has fewer links than its minimum (which must be at least one).
fn ghosts_for(graph: &Scope, inbound: &Inbound, node: &Node, rule: &str, check: &Obj) -> Vec<Ghost> {
    let Some(list) = check.get("require_link") else { return vec![] };
    let mut out = Vec::new();
    for it in items(list) {
        let Some(it) = it.as_object().filter(|it| link_item_valid(it) && dir_of(it) == Some("out")) else { continue };
        let (lo, _) = link_bounds(it);
        if lo >= 1 && count_links(graph, inbound, node, it) < lo {
            let get = |a: &str, b: &str| it.get(a).or_else(|| it.get(b)).cloned().unwrap_or(Value::Null);
            let plane = get("node_plane", "to_plane");
            out.push(Ghost {
                node: node.id.clone(),
                rule: rule.to_string(),
                link: it["type"].clone(),
                to_type: get("node_type", "to_type"),
                plane: if truthy(&plane) { plane } else { opt_value(&node.plane) },
            });
        }
    }
    out
}

/// Run every rule over the graph.
pub fn evaluate(graph: &Graph) -> Evaluation {
    evaluate_with(graph, &crate::derive::derive_all(graph))
}

/// The calculated values rules can read: each node's derived values as plain values (see [`Calc`]).
fn calc_of(derived: &crate::derive::DerivedMap) -> BTreeMap<&str, Calc> {
    use crate::derive::Derived;
    let mut out = BTreeMap::new();
    for (id, row) in derived {
        let mut calc = Calc::new();
        for (name, d) in row {
            let values = match d {
                Derived::Values(v) | Derived::Overridden { values: v, .. } => v,
                Derived::Missing(_) => continue,
            };
            match values.as_slice() {
                [] => {}
                [one] => {
                    calc.insert(name.clone(), Value::String(one.clone()));
                }
                many => {
                    calc.insert(name.clone(), Value::Array(many.iter().cloned().map(Value::String).collect()));
                }
            }
        }
        if !calc.is_empty() {
            out.insert(id.as_str(), calc);
        }
    }
    out
}

/// [`evaluate`] with the derived values already worked out, so `check` computes them once.
pub fn evaluate_with(graph: &Graph, derived: &crate::derive::DerivedMap) -> Evaluation {
    let calcs = calc_of(derived);
    let rules: Vec<&Node> = graph.nodes().filter(|n| n.kind.as_deref() == Some("rule")).collect();
    let inbound = build_inbound(graph);
    let names = Names::new(graph);
    let mut out = Evaluation::default();
    let empty = Obj::new();

    for node in graph.nodes() {
        if node.kind.as_deref().is_some_and(|k| RESERVED.contains(&k)) || graph.scratch_of(node).is_some() {
            continue;
        }
        for rule in &rules {
            // A rule with `enabled: false` is switched off: no results, no ghosts.
            if rule.fields.get("enabled") == Some(&Value::Bool(false)) {
                continue;
            }
            // `applies_to` that is missing or not a map matches every node.
            let filter = rule.fields.get("applies_to").and_then(Value::as_object).unwrap_or(&empty);
            // the rule's names are read from its own pack: its own types first, then those of the packs it requires
            let scope = Scope::new(graph, &names, def_ns(&rule.id));
            if !matches(&scope, node, filter) {
                continue;
            }
            let severity = match rule.fields.get("severity").and_then(Value::as_str) {
                Some(s @ ("error" | "warn" | "info")) => s.to_string(),
                _ => "warn".to_string(),
            };
            let check = rule.fields.get("check").and_then(Value::as_object).filter(|c| !c.is_empty());
            let status = match check {
                None => Status::Advisory,
                Some(c) => run_check(&scope, &inbound, node, calcs.get(node.id.as_str()), c),
            };
            out.results.push(RuleResult { node: node.id.clone(), rule: rule.id.clone(), severity, status });
            if let (Status::Fail, Some(c)) = (status, check) {
                out.ghosts.extend(ghosts_for(&scope, &inbound, node, &rule.id, c));
            }
        }
    }
    // Nodes and rules both come out in id order, so results and ghosts are already sorted by (node, rule).
    debug_assert!(out.results.windows(2).all(|w| (&w[0].node, &w[0].rule) <= (&w[1].node, &w[1].rule)));
    out
}

/// Enabled rules that name specific nodes with `id` but match none of them: a typo looks exactly like success.
pub fn unmatched_rules(graph: &Graph) -> Vec<String> {
    let names = Names::new(graph);
    let mut out = Vec::new();
    for r in graph.nodes().filter(|n| n.kind.as_deref() == Some("rule")) {
        if r.fields.get("enabled") == Some(&Value::Bool(false)) {
            continue;
        }
        let Some(filter) = r.fields.get("applies_to").and_then(Value::as_object).filter(|f| f.contains_key("id"))
        else {
            continue;
        };
        let scope = Scope::new(graph, &names, def_ns(&r.id));
        let hit = graph.nodes().any(|n| {
            !n.kind.as_deref().is_some_and(|k| RESERVED.contains(&k))
                && graph.scratch_of(n).is_none()
                && matches(&scope, n, filter)
        });
        if !hit {
            out.push(r.id.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn equality_is_json_equality() {
        assert!(json_eq(&json!(1), &json!(1.0)));
        assert!(!json_eq(&json!(true), &json!(1)));
        assert!(!json_eq(&json!([1, true]), &json!([true, 1])));
        assert!(json_eq(&json!({"a": [1, 2.0]}), &json!({"a": [1.0, 2]})));
        assert!(!json_eq(&json!(null), &json!(0)));
        assert!(!json_eq(&json!("1"), &json!(1)));
        assert!(!json_eq(&json!(9007199254740993i64), &json!(9007199254740992i64)));
    }

    #[test]
    fn truthiness() {
        assert!(!truthy(&json!("")) && !truthy(&json!(null)) && !truthy(&json!(0)));
        assert!(truthy(&json!("p")) && truthy(&json!([0])));
    }
}
