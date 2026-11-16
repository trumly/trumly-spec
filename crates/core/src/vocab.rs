//! What a project's vocabulary is, read from the project itself. A project's planes, node types, link types, rules and packs are
//! ordinary nodes (`plane`, `type`, `link-type`, `rule`, `pack`), so everything an agent needs to know about what it may
//! create is already in the files. This turns it into answers: the `describe_types` tool, the "what you can create"
//! section of `get_index`, the project overview, and the advice and defaults a write gets when a node strays from the
//! vocabulary. Nothing here is enforced: the checker never fails on an undefined type, so this only informs and advises.
//!
//! A `type` node may say where its nodes belong with `fields.plane` (the id of a `plane`). Without it the plane is
//! inferred from where existing nodes of that type sit.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::model::Node;
use crate::report::Report;

/// The id of the overview node: by convention the first thing a new reader should see.
pub const OVERVIEW_ID: &str = "overview";

/// The questions an overview answers, in the order they are shown: (field, label, what it holds). These are the fields of the `about` pack's
/// `overview` type; any other field on the node is shown after them.
pub const OVERVIEW_FIELDS: [(&str, &str, &str); 5] = [
    ("purpose", "Purpose", "what is being built and for whom, in a sentence or two"),
    ("audience", "Read by", "who reads this design and acts on it: people, agents, which team"),
    ("owner", "Owner", "who keeps it right, and who to ask"),
    ("stage", "Stage", "define, design, plan or build: which phase the project is in"),
    (
        "source-of-truth",
        "Source of truth",
        "where the code, backlog and other documents live, and which wins when they disagree",
    ),
];

/// The type the `about` pack defines for a project's own conventions: how it is written, one node per rule of thumb.
pub const CONVENTION_TYPE: &str = "convention";

const RESERVED: [&str; 6] = ["plane", "type", "link-type", "rule", "pack", "group"];

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}
fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}
fn fstr(n: &Node, key: &str) -> String {
    n.fields.get(key).map(s).unwrap_or_default()
}
fn kind(n: &Node) -> &str {
    n.kind.as_deref().unwrap_or("")
}
fn union(lists: impl Iterator<Item = Vec<String>>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for l in lists {
        for x in l {
            if seen.insert(x.clone()) {
                out.push(x);
            }
        }
    }
    out
}

/// A project's vocabulary: its definition nodes, and how its ordinary nodes are spread over types and planes.
struct Vocab<'a> {
    packs: Vec<&'a Node>,
    planes: Vec<&'a Node>,
    types: BTreeMap<String, Vec<&'a Node>>,
    link_types: BTreeMap<String, Vec<&'a Node>>,
    rules: Vec<&'a Node>,
    counts: BTreeMap<String, usize>,
    planes_of: BTreeMap<String, BTreeMap<String, usize>>,
}

fn vocab(report: &Report) -> Vocab<'_> {
    let mut v = Vocab {
        packs: vec![],
        planes: vec![],
        types: BTreeMap::new(),
        link_types: BTreeMap::new(),
        rules: vec![],
        counts: BTreeMap::new(),
        planes_of: BTreeMap::new(),
    };
    for n in report.graph.nodes() {
        let name = |n: &Node| Some(fstr(n, "name")).filter(|x| !x.is_empty()).unwrap_or_else(|| n.title.clone());
        match kind(n) {
            "pack" => v.packs.push(n),
            "plane" => v.planes.push(n),
            "type" => v.types.entry(name(n)).or_default().push(n),
            "link-type" => v.link_types.entry(name(n)).or_default().push(n),
            "rule" => v.rules.push(n),
            k => {
                *v.counts.entry(k.to_string()).or_default() += 1;
                if let Some(p) = &n.plane {
                    *v.planes_of.entry(k.to_string()).or_default().entry(p.clone()).or_default() += 1;
                }
            }
        }
    }
    let order = |n: &Node| n.fields.get("order").and_then(Value::as_f64).unwrap_or(0.0);
    v.planes.sort_by(|a, b| order(a).partial_cmp(&order(b)).unwrap_or(std::cmp::Ordering::Equal).then(a.id.cmp(&b.id)));
    v
}

/// Where nodes of a type belong: the plane its definition declares, and the planes existing nodes sit on (most first).
fn plane_for(name: &str, defs: &[&Node], v: &Vocab) -> (Option<String>, Vec<String>) {
    let declared = defs.iter().map(|d| fstr(d, "plane")).find(|p| !p.is_empty());
    let mut seen: Vec<(&String, &usize)> = v.planes_of.get(name).map(|m| m.iter().collect()).unwrap_or_default();
    seen.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    (declared, seen.into_iter().map(|(p, _)| p.clone()).collect())
}

fn field_defs(n: &Node) -> Vec<Value> {
    n.fields
        .get("fields")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|f| {
                    let name = f.get("name")?.as_str()?.to_string();
                    if name.is_empty() {
                        return None; // a field still being named in an editor
                    }
                    let mut o = Map::new();
                    o.insert("name".into(), json!(name));
                    o.insert("kind".into(), json!(f.get("kind").and_then(Value::as_str).unwrap_or("text")));
                    if let Some(l) = f.get("label").and_then(Value::as_str) {
                        o.insert("label".into(), json!(l));
                    }
                    if f.get("required") == Some(&Value::Bool(true)) {
                        o.insert("required".into(), json!(true));
                    }
                    for k in ["items", "fields"] {
                        if let Some(x) = f.get(k).filter(|x| x.is_string() || x.is_array()) {
                            o.insert(k.into(), x.clone());
                        }
                    }
                    if let Some(vs) = f.get("values").and_then(Value::as_array) {
                        o.insert(
                            "values".into(),
                            json!(
                                vs.iter()
                                    .map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_string))
                                    .collect::<Vec<_>>()
                            ),
                        );
                    }
                    Some(Value::Object(o))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn describe_type(name: &str, defs: &[&Node], v: &Vocab) -> Value {
    let mut merged: Vec<Value> = Vec::new();
    for d in defs {
        for f in field_defs(d) {
            match merged.iter_mut().find(|m| m["name"] == f["name"]) {
                Some(m) => {
                    let req = m.get("required") == Some(&json!(true)) || f.get("required") == Some(&json!(true));
                    for (k, x) in f.as_object().unwrap() {
                        m[k] = x.clone();
                    }
                    if req {
                        m["required"] = json!(true);
                    }
                }
                None => merged.push(f),
            }
        }
    }
    let main = defs[0];
    let (declared, seen) = plane_for(name, defs, v);
    let mut links_out = vec![];
    let mut links_in = vec![];
    for (ln, lds) in &v.link_types {
        let from = union(lds.iter().map(|l| strs(l.fields.get("from"))));
        let to = union(lds.iter().map(|l| strs(l.fields.get("to"))));
        if from.is_empty() || from.iter().any(|x| x == name) {
            links_out.push(json!({"link": ln, "to": to}));
        }
        if to.is_empty() || to.iter().any(|x| x == name) {
            links_in.push(json!({"link": ln, "from": from}));
        }
    }
    let rules: Vec<Value> = v
        .rules
        .iter()
        .filter(|r| match r.fields.get("applies_to").and_then(|a| a.get("type")) {
            Some(Value::String(t)) => t == name,
            Some(Value::Array(list)) => list.iter().any(|t| t.as_str() == Some(name)),
            _ => false,
        })
        .map(|r| json!({"id": r.id, "statement": Some(fstr(r, "statement")).filter(|x| !x.is_empty()).unwrap_or_else(|| r.title.clone()), "severity": Some(fstr(r, "severity")).filter(|x| !x.is_empty()).unwrap_or_else(|| "warn".into())}))
        .collect();
    let required: Vec<Value> =
        merged.iter().filter(|f| f["required"] == json!(true)).map(|f| f["name"].clone()).collect();
    let plane = declared.clone().or_else(|| seen.first().cloned());
    let example_fields: Map<String, Value> = merged
        .iter()
        .filter(|f| f["required"] == json!(true))
        .map(|f| {
            let first = f.get("values").and_then(|x| x.get(0)).cloned();
            let blank = match f["kind"].as_str() {
                Some("boolean") => json!(false),
                Some("number") => json!(0),
                Some("list") => json!([]),
                Some("object") => json!({}),
                _ => json!(""),
            };
            (f["name"].as_str().unwrap_or("").to_string(), first.unwrap_or(blank))
        })
        .collect();
    let description =
        Some(fstr(main, "description")).filter(|d| !d.is_empty()).unwrap_or_else(|| main.body.trim().to_string());
    let mut out = json!({
        "name": name, "description": description, "plane": plane, "existing_nodes": v.counts.get(name).copied().unwrap_or(0),
        "fields": merged, "required_fields": required, "links_out": links_out, "links_in": links_in, "rules": rules,
        "id_convention": format!("{name}-<short-name>"),
        "example": {"id": format!("{name}-example"), "type": name, "title": "Example", "parent": null, "plane": plane, "fields": example_fields, "links": [], "body": ""},
    });
    for k in ["label", "pack", "shape"] {
        if let Some(x) = main.fields.get(k).filter(|x| !x.is_null()) {
            out[k] = x.clone();
        }
    }
    if let Some(pk) = main.fields.get("pack") {
        out["pack"] = pk.clone();
    }
    if !seen.is_empty() {
        out["seen_on_planes"] = json!(seen);
    }
    out
}

fn plane_summary(p: &Node, v: &Vocab) -> Value {
    let f = &p.fields;
    let nodes: usize = v.planes_of.values().map(|m| m.get(&p.id).copied().unwrap_or(0)).sum();
    let mut o = json!({"id": p.id, "title": p.title, "order": f.get("order").and_then(Value::as_f64).unwrap_or(0.0), "nodes": nodes});
    for k in ["pack", "stack"] {
        if let Some(x) = f.get(k).filter(|x| !x.is_null()) {
            o[k] = x.clone();
        }
    }
    if f.get("scratch") == Some(&Value::Bool(true)) {
        o["scratch"] = json!(true);
    }
    if !p.body.trim().is_empty() {
        o["description"] = json!(p.body.trim());
    }
    o
}

/// The `describe_types` answer: the vocabulary of the project, or one slice of it (`type`, `plane` or `pack`).
pub fn describe_types(report: &Report, args: &Value) -> Value {
    let v = vocab(report);
    let (a_type, a_plane, a_pack) = (args["type"].as_str(), args["plane"].as_str(), args["pack"].as_str());
    let in_pack = |n: &Node| a_pack.is_none_or(|p| fstr(n, "pack").split('@').next() == Some(p));
    let types: Vec<Value> =
        v.types.iter().filter(|(_, d)| d.iter().any(|n| in_pack(n))).map(|(n, d)| describe_type(n, d, &v)).collect();
    let picked: Vec<Value> = match (a_type, a_plane) {
        (Some(t), _) => types.iter().filter(|x| x["name"] == t).cloned().collect(),
        (None, Some(p)) => types
            .iter()
            .filter(|x| x["plane"] == p || x["seen_on_planes"].as_array().is_some_and(|a| a.iter().any(|y| y == p)))
            .cloned()
            .collect(),
        _ => types.clone(),
    };
    let packs: Vec<Value> = v
        .packs
        .iter()
        .filter(|p| a_pack.is_none_or(|n| fstr(p, "name") == n))
        .map(|p| {
            let mut o = json!({"id": p.id, "name": fstr(p, "name"), "version": fstr(p, "version"), "description": Some(fstr(p, "description")).filter(|d| !d.is_empty()).unwrap_or_else(|| p.body.trim().to_string())});
            if let Some(ns) = p.fields.get("namespace") {
                o["namespace"] = ns.clone();
            }
            let req = strs(p.fields.get("requires"));
            if !req.is_empty() {
                o["requires"] = json!(req);
            }
            o
        })
        .collect();
    let mut out = json!({
        "note": "The checker does not reject a type or link type that is not defined here, so use these names exactly: nodes of an undefined type work but nothing knows how to draw, validate or advise on them.",
        "packs": packs, "planes": v.planes.iter().map(|p| plane_summary(p, &v)).collect::<Vec<_>>(), "types": picked,
    });
    if a_type.is_none() && a_plane.is_none() && a_pack.is_none() {
        out["overview"] = json!({
            "id": OVERVIEW_ID, "exists": report.graph.get(OVERVIEW_ID).is_some(),
            "note": "The project says what it is for in one node with the id `overview` (type `overview`, from the `about` pack): its fields and body are shown at the top of get_index. How the project is written goes in nodes of type `convention`, which can point at a plane, type or node with `describes`.",
            "convention_type": CONVENTION_TYPE, "has_convention_type": v.types.contains_key(CONVENTION_TYPE), "has_conventions": report.graph.nodes().any(|n| kind(n) == CONVENTION_TYPE),
            "fields": OVERVIEW_FIELDS.iter().map(|(n, l, h)| json!({"name": n, "label": l, "about": h})).collect::<Vec<_>>(),
        });
    }
    if let Some(t) = a_type.filter(|_| picked_empty(&out)) {
        let near = suggest(t, &v.types.keys().cloned().collect::<Vec<_>>(), 3);
        out["error"] = json!(format!(
            "no type named '{t}'{}",
            if near.is_empty() {
                String::new()
            } else {
                format!("; did you mean {}?", near.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", "))
            }
        ));
    }
    if a_type.is_none() {
        let lt: Vec<Value> = v
            .link_types
            .iter()
            .filter(|(_, d)| d.iter().any(|n| in_pack(n)))
            .map(|(n, d)| {
                let mut o = json!({"name": n, "description": Some(fstr(d[0], "description")).filter(|x| !x.is_empty()).unwrap_or_else(|| d[0].body.trim().to_string()), "from": union(d.iter().map(|l| strs(l.fields.get("from")))), "to": union(d.iter().map(|l| strs(l.fields.get("to"))))});
                if let Some(i) = d[0].fields.get("inverse") {
                    o["inverse"] = i.clone();
                }
                o
            })
            .collect();
        out["link_types"] = json!(lt);
        out["rules"] = json!(v
            .rules
            .iter()
            .filter(|r| in_pack(r))
            .map(|r| json!({"id": r.id, "statement": Some(fstr(r, "statement")).filter(|x| !x.is_empty()).unwrap_or_else(|| r.title.clone()), "severity": Some(fstr(r, "severity")).filter(|x| !x.is_empty()).unwrap_or_else(|| "warn".into()), "applies_to": r.fields.get("applies_to").cloned().unwrap_or(Value::Null)}))
            .collect::<Vec<_>>());
    }
    out
}

fn picked_empty(out: &Value) -> bool {
    out["types"].as_array().is_some_and(Vec::is_empty)
}

fn ov_text(v: Option<&Value>) -> String {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| format!("- {}", x.as_str().map_or_else(|| x.to_string(), str::to_string)))
            .collect::<Vec<_>>()
            .join("\n"),
        Some(Value::String(t)) => t.trim().to_string(),
        Some(Value::Object(o)) => o
            .iter()
            .map(|(k, x)| format!("- **{k}**: {}", x.as_str().map_or_else(|| x.to_string(), str::to_string)))
            .collect::<Vec<_>>()
            .join("\n"),
        Some(Value::Null) | None => String::new(),
        Some(x) => x.to_string(),
    }
}

/// The overview as `(title, text)`: its named fields in order, any other fields after, then the body.
pub fn overview(report: &Report) -> Option<(String, String)> {
    let n = report.graph.get(OVERVIEW_ID)?;
    let known: BTreeSet<&str> = OVERVIEW_FIELDS.iter().map(|f| f.0).collect();
    let mut parts: Vec<String> = Vec::new();
    for (name, label, _) in OVERVIEW_FIELDS {
        let t = ov_text(n.fields.get(name));
        if t.is_empty() {
            continue;
        }
        parts.push(if n.fields.get(name).is_some_and(Value::is_array) || t.contains('\n') {
            format!("**{label}**\n{t}")
        } else {
            format!("**{label}:** {t}")
        });
    }
    for (k, x) in &n.fields {
        let t = ov_text(Some(x));
        if !known.contains(k.as_str()) && !t.is_empty() {
            parts.push(format!("**{k}:** {t}"));
        }
    }
    let text = [parts.join("\n\n"), n.body.trim().to_string()]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    Some((n.title.clone(), text))
}

/// The project's conventions, how it is written: its `convention` nodes as a list of title and body, each saying what it describes.
pub fn conventions(report: &Report) -> Option<String> {
    let mut nodes: Vec<&Node> = report.graph.nodes().filter(|n| kind(n) == CONVENTION_TYPE).collect();
    if nodes.is_empty() {
        return None;
    }
    nodes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()).then(a.id.cmp(&b.id)));
    let items: Vec<String> = nodes
        .iter()
        .map(|n| {
            let about: Vec<&str> = n.links.iter().filter(|l| l.kind == "describes").map(|l| l.to.as_str()).collect();
            let tail = if about.is_empty() {
                String::new()
            } else {
                format!(" _(describes {})_", about.iter().map(|a| format!("`{a}`")).collect::<Vec<_>>().join(", "))
            };
            let body = n.body.trim().replace('\n', "\n  ");
            if body.is_empty() {
                format!("- **{}**{tail}", n.title)
            } else {
                format!("- **{}**{tail}: {body}", n.title)
            }
        })
        .collect();
    Some(format!("**Conventions**\n{}", items.join("\n")))
}

/// The index an agent reads first: the project's own overview, then the map of the project, then what can be created where.
pub fn agent_index(report: &Report, write: bool) -> String {
    let v = vocab(report);
    let mut head = String::new();
    match overview(report) {
        Some((title, text)) => head.push_str(&format!("# About this project: {title}\n\n{}\n\n---\n\n", if text.is_empty() { "_The overview node is empty._" } else { &text })),
        None if write => head.push_str(&format!(
            "> This project has no overview yet. If you learn what it is for, write it down: create a node with id `{OVERVIEW_ID}` (type `overview`) and fill in its fields: {}. The body takes anything else. It is shown here to everyone who connects.\n\n",
            OVERVIEW_FIELDS.iter().map(|f| f.0).collect::<Vec<_>>().join(", ")
        )),
        None => {}
    }
    if let Some(text) = conventions(report) {
        head.push_str(&format!("{text}\n\n---\n\n"));
    }
    let mut index = report.index();
    while index.ends_with('\n') {
        index.pop();
    }
    index.push('\n');
    if v.types.is_empty() && v.planes.is_empty() {
        return head + &index;
    }
    let mut lines = vec![String::new(), "## What you can create".to_string(), String::new(), "Call `describe_types` for fields, links, rules and an example of any type. Types by plane (counts are existing nodes):".to_string(), String::new()];
    let mut placed = BTreeSet::new();
    let count = |n: &str| v.counts.get(n).copied().unwrap_or(0);
    for p in &v.planes {
        let here: Vec<&String> = v
            .types
            .iter()
            .filter(|(n, d)| {
                let (declared, seen) = plane_for(n, d, &v);
                declared.as_deref() == Some(p.id.as_str()) || (declared.is_none() && seen.first() == Some(&p.id))
            })
            .map(|(n, _)| n)
            .collect();
        placed.extend(here.iter().map(|n| n.to_string()));
        let scratch = if p.fields.get("scratch") == Some(&Value::Bool(true)) { " (scratch)" } else { "" };
        let list = if here.is_empty() {
            "no type is tied to this plane; see describe_types".to_string()
        } else {
            here.iter().map(|n| format!("`{n}` ({})", count(n))).collect::<Vec<_>>().join(", ")
        };
        lines.push(format!("- `{}` {}{scratch}: {list}", p.id, p.title));
    }
    let rest: Vec<String> =
        v.types.keys().filter(|n| !placed.contains(*n)).map(|n| format!("`{n}` ({})", count(n))).collect();
    if !rest.is_empty() {
        lines.push(format!("- no plane recorded: {}", rest.join(", ")));
    }
    if !v.link_types.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Link types: {}",
            v.link_types.keys().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")
        ));
    }
    head + &index + &lines.join("\n") + "\n"
}

/// Closest names, for "did you mean".
pub fn suggest(name: &str, known: &[String], max: usize) -> Vec<String> {
    let n = name.to_lowercase();
    let score = |k: &str| -> f64 {
        let x = k.to_lowercase();
        if x == n {
            0.0
        } else if x.starts_with(&n) || n.starts_with(&x) {
            1.0
        } else if x.contains(&n) || n.contains(&x) {
            2.0
        } else if is_subsequence(&n, &x) {
            3.0
        } else {
            2.0 + dist(&n, &x) as f64 / n.chars().count().max(x.chars().count()).max(1) as f64 * 10.0
        }
    };
    let mut scored: Vec<(&String, f64)> = known.iter().map(|k| (k, score(k))).filter(|(_, sc)| *sc <= 6.0).collect();
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(max).map(|(k, _)| k.clone()).collect()
}

/// Every character of `short` appears in `long`, in order (`srv` in `server`).
fn is_subsequence(short: &str, long: &str) -> bool {
    let mut it = long.chars();
    short.chars().all(|c| it.any(|d| d == c))
}

fn dist(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + usize::from(a[i - 1] != b[j - 1]));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Get a node ready to write: fill in what the vocabulary already knows, and say what looks wrong.
///
/// `node` is the node an agent passed; `is_new` says it has no earlier version. A new node of a type that says which
/// plane it belongs on (or that always sits on one plane) and names none gets that plane. Returns the node to write,
/// what was filled in, and advice. The advice never blocks a write.
pub fn prepare(report: &Report, node: &Value, is_new: bool) -> Value {
    let v = vocab(report);
    let mut node = node.clone();
    let mut defaulted = Map::new();
    let mut advice: Vec<String> = vec![];
    let ty = node["type"].as_str().unwrap_or("").to_string();
    let id = node["id"].as_str().unwrap_or("").to_string();
    if !RESERVED.contains(&ty.as_str()) && !ty.is_empty() {
        if !v.types.is_empty() && !v.types.contains_key(&ty) {
            let near = suggest(&ty, &v.types.keys().cloned().collect::<Vec<_>>(), 3);
            advice.push(format!(
                "unknown_type: no type named '{ty}' is defined in this project{} Call describe_types to see the types; change the type with another put_node (same id, pass the rev you got back).",
                if near.is_empty() { ".".to_string() } else { format!(". Did you mean {}?", near.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", ")) }
            ));
        }
        if let Some(defs) = v.types.get(&ty) {
            let (declared, seen) = plane_for(&ty, defs, &v);
            let home = declared.clone().or_else(|| seen.first().cloned());
            let plane = node["plane"].as_str().filter(|p| !p.is_empty()).map(str::to_string);
            match (&plane, &home) {
                (None, Some(h))
                    if is_new && (declared.is_some() || seen.len() == 1) && v.planes.iter().any(|p| &p.id == h) =>
                {
                    node["plane"] = json!(h);
                    defaulted.insert("plane".into(), json!(h));
                }
                (None, Some(h)) => advice
                    .push(format!("no_plane: nodes of type '{ty}' normally sit on plane '{h}'; this one has none.")),
                (Some(p), Some(h)) if p != h && !seen.contains(p) => {
                    advice.push(format!("unusual_plane: nodes of type '{ty}' normally sit on '{h}', not '{p}'."))
                }
                _ => {}
            }
        }
        if let Some(p) = node["plane"].as_str().filter(|p| !p.is_empty()) {
            if !v.planes.is_empty() && !v.planes.iter().any(|x| x.id == p) {
                advice.push(format!(
                    "unknown_plane: '{p}' is not a plane. Planes: {}.",
                    v.planes.iter().map(|x| format!("'{}'", x.id)).collect::<Vec<_>>().join(", ")
                ));
            }
        }
        if !v.link_types.is_empty() {
            for l in node["links"].as_array().into_iter().flatten() {
                if let Some(t) = l["type"].as_str().filter(|t| !v.link_types.contains_key(*t)) {
                    let near = suggest(t, &v.link_types.keys().cloned().collect::<Vec<_>>(), 3);
                    advice.push(format!(
                        "unknown_link_type: no link type named '{t}'{}",
                        if near.is_empty() {
                            String::new()
                        } else {
                            format!(
                                ". Did you mean {}?",
                                near.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", ")
                            )
                        }
                    ));
                }
            }
        }
        if !id.contains("--") && !id.starts_with(&format!("{ty}-")) && id != OVERVIEW_ID && !id.is_empty() {
            let rest = id.split_once('-').map_or(id.as_str(), |x| x.1);
            advice.push(format!("id_convention: ids are usually '<type>-<name>' (e.g. '{ty}-{rest}'). Not checked; rename_node can fix it."));
        }
    }
    json!({"node": node, "defaulted": Value::Object(defaulted), "advice": advice})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::check;

    fn model(extra: &[(&str, &str)]) -> Report {
        let base: Vec<(&str, &str)> = vec![
            (
                "dm--pack.md",
                "---\nid: dm--pack\ntype: pack\nfields: {name: data-model, version: 0.3.0, namespace: dm, description: Concept to table}\n---\n",
            ),
            (
                "dm--physical.md",
                "---\nid: dm--physical\ntype: plane\ntitle: Physical\nfields: {order: 3, pack: data-model@0.3.0}\n---\nTables and columns.\n",
            ),
            ("dm--logical.md", "---\nid: dm--logical\ntype: plane\ntitle: Logical\nfields: {order: 2}\n---\n"),
            (
                "dm--type-data-table.md",
                "---\nid: dm--type-data-table\ntype: type\nfields: {name: data-table, description: A table, pack: data-model@0.3.0, fields: [{name: rows, kind: number, required: true}]}\n---\n",
            ),
            (
                "dm--type-data-db-server.md",
                "---\nid: dm--type-data-db-server\ntype: type\nfields: {name: data-db-server, description: A server, plane: dm--physical, fields: []}\n---\n",
            ),
            (
                "dm--link-type-data-contains.md",
                "---\nid: dm--link-type-data-contains\ntype: link-type\nfields: {name: data-contains, from: [data-db-server], to: [data-table], inverse: contained-by}\n---\n",
            ),
            (
                "dm--rule-table-pk.md",
                "---\nid: dm--rule-table-pk\ntype: rule\nfields: {statement: Every table has a key, severity: warn, applies_to: {type: data-table}}\n---\n",
            ),
            (
                "data-table-orders.md",
                "---\nid: data-table-orders\ntype: data-table\nplane: dm--physical\nfields: {rows: 3}\n---\n",
            ),
        ];
        check(base.iter().chain(extra.iter()).map(|(p, t)| (p.to_string(), t.to_string())).collect())
    }

    #[test]
    fn describes_planes_types_links_rules_and_an_example() {
        let r = describe_types(&model(&[]), &json!({}));
        let planes: Vec<&str> = r["planes"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
        assert_eq!(planes, ["dm--logical", "dm--physical"]);
        assert_eq!(r["packs"][0]["name"], "data-model");
        let t = r["types"].as_array().unwrap().iter().find(|x| x["name"] == "data-table").unwrap();
        assert_eq!(t["plane"], "dm--physical");
        assert_eq!(t["existing_nodes"], 1);
        assert_eq!(t["required_fields"], json!(["rows"]));
        assert_eq!(t["links_in"], json!([{"link": "data-contains", "from": ["data-db-server"]}]));
        assert_eq!(t["rules"][0]["statement"], "Every table has a key");
        assert_eq!(t["example"]["fields"]["rows"], 0);
        let s = r["types"].as_array().unwrap().iter().find(|x| x["name"] == "data-db-server").unwrap();
        assert_eq!(s["plane"], "dm--physical");
        assert_eq!(r["link_types"][0]["inverse"], "contained-by");
        assert!(r["overview"]["fields"].as_array().unwrap().iter().any(|f| f["name"] == "purpose"));
    }

    #[test]
    fn narrows_and_suggests() {
        let m = model(&[]);
        assert_eq!(describe_types(&m, &json!({"type": "data-table"}))["types"].as_array().unwrap().len(), 1);
        let by_plane = describe_types(&m, &json!({"plane": "dm--physical"}));
        let on: Vec<&str> = by_plane["types"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
        assert_eq!(on, ["data-db-server", "data-table"]);
        assert!(
            describe_types(&m, &json!({"type": "data-server"}))["error"].as_str().unwrap().contains("data-db-server")
        );
    }

    #[test]
    fn index_opens_with_the_overview_and_ends_with_what_can_be_created() {
        let ov = "---\nid: overview\ntype: overview\ntitle: Orders\nfields: {audience: agents, purpose: The orders DB, stage: design, extra: x}\n---\nMore.\n";
        let i = agent_index(&model(&[("overview.md", ov)]), true);
        assert!(i.starts_with("# About this project: Orders"));
        assert!(i.contains(
            "**Purpose:** The orders DB\n\n**Read by:** agents\n\n**Stage:** design\n\n**extra:** x\n\nMore."
        ));
        assert!(i.contains("`dm--physical` Physical: `data-db-server` (0), `data-table` (1)"), "{i}");
        assert!(i.contains("Link types: `data-contains`"));
        assert!(agent_index(&model(&[]), true).contains("no overview yet"));
        assert!(!agent_index(&model(&[]), false).contains("no overview yet"));
    }

    #[test]
    fn conventions_follow_the_overview_in_the_index_and_say_what_they_describe() {
        let m = model(&[
            (
                "overview.md",
                "---\nid: overview\ntype: overview\ntitle: Orders\nfields: {purpose: The orders DB}\n---\n",
            ),
            (
                "convention-short.md",
                "---\nid: convention-short\ntype: convention\ntitle: Short titles\n---\nTitles stay under thirty characters.\n",
            ),
            (
                "convention-k8s.md",
                "---\nid: convention-k8s\ntype: convention\ntitle: Kubernetes layer\nlinks:\n  - type: describes\n    to: kubernetes\n---\n",
            ),
        ]);
        let i = agent_index(&m, true);
        assert!(i.starts_with("# About this project: Orders\n\n**Purpose:** The orders DB\n\n---\n\n**Conventions**\n- **Kubernetes layer** _(describes `kubernetes`)_\n- **Short titles**: Titles stay under thirty characters.\n\n---"), "{i}");
        assert!(!i.contains("no overview yet"));
        assert!(conventions(&model(&[])).is_none());
        // with no overview and no conventions an agent that can write is told how to start
        assert!(agent_index(&model(&[]), true).contains("create a node with id `overview` (type `overview`)"));
    }

    #[test]
    fn prepare_fills_the_plane_and_advises() {
        let m = model(&[]);
        let p = prepare(&m, &json!({"id": "data-db-server-main", "type": "data-db-server", "links": []}), true);
        assert_eq!(p["node"]["plane"], "dm--physical");
        assert_eq!(p["defaulted"], json!({"plane": "dm--physical"}));
        assert_eq!(p["advice"], json!([]));
        // replacing never invents a plane
        let q = prepare(&m, &json!({"id": "data-db-server-main", "type": "data-db-server"}), false);
        assert!(q["node"]["plane"].is_null());
        assert!(q["advice"][0].as_str().unwrap().starts_with("no_plane"));
        let a = prepare(
            &m,
            &json!({"id": "data-server-new", "type": "data-server", "plane": "dm--physical", "links": [{"type": "contains", "to": "x"}]}),
            true,
        );
        let text = a["advice"].to_string();
        assert!(
            text.contains("unknown_type")
                && text.contains("data-db-server")
                && text.contains("unknown_link_type")
                && text.contains("data-contains"),
            "{text}"
        );
        assert!(
            prepare(&m, &json!({"id": "data-table-a", "type": "data-table", "plane": "nowhere"}), true)["advice"]
                .to_string()
                .contains("unknown_plane")
        );
        assert!(
            prepare(&m, &json!({"id": "thing", "type": "data-table", "plane": "dm--physical"}), true)["advice"]
                .to_string()
                .contains("id_convention")
        );
        assert_eq!(
            prepare(
                &check(vec![("a.md".into(), "---\nid: a\ntype: component\n---\n".into())]),
                &json!({"id": "component-b", "type": "component"}),
                true
            )["advice"],
            json!([])
        );
    }

    #[test]
    fn suggests_by_prefix_containment_and_distance() {
        let known = ["data-db-server".to_string(), "data-table".to_string(), "component".to_string()];
        assert_eq!(suggest("data-server", &known, 3)[0], "data-db-server");
    }
}
