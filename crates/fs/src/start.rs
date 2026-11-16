//! Starting a project: the short questions a person is asked first (the same ones the cloud's guided start asks), and
//! what the answers become: the core bundle, any extra packs chosen, and the `overview` node that says what the project is
//! for. The questions are data in `spec/start-questions.json`, so every surface asks the same thing.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use trumly_core::format::node_to_markdown;
use trumly_core::model::Node;
use trumly_core::parse::parse_node;

use crate::pack::{Clash, Pack, add_tree_with, installed};

const QUESTIONS: &str = include_str!("../../../spec/start-questions.json");
pub const APPROACHES: [&str; 2] = ["greenfield", "brownfield"];
const LIMIT: usize = 300;

/// The questions, as JSON. `packs` fills in the extra packs on offer.
pub fn questions(packs: &[(String, String)]) -> Value {
    let mut q: Value = serde_json::from_str(QUESTIONS).expect("start-questions.json is valid");
    for item in q["questions"].as_array_mut().into_iter().flatten() {
        if item["options_from"] == "packs" {
            item["options"] =
                Value::Array(packs.iter().map(|(n, d)| json!({"value": n, "label": n, "hint": d})).collect());
        }
    }
    q
}

#[derive(Debug, Default, Clone)]
pub struct Answers {
    pub name: String,
    pub purpose: String,
    pub audience: String,
    pub owner: String,
    pub approach: String,
    pub packs: Vec<String>,
}

fn text(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).map(|s| s.split_whitespace().collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

/// Read answers out of a JSON object. Returns the answers and the names of the required ones still missing.
pub fn read_answers(v: &Value) -> (Answers, Vec<&'static str>) {
    let a = Answers {
        name: text(v, "name"),
        purpose: text(v, "purpose"),
        audience: text(v, "audience"),
        owner: text(v, "owner"),
        approach: text(v, "approach"),
        packs: v
            .get("packs")
            .and_then(Value::as_array)
            .map(|p| p.iter().filter_map(Value::as_str).map(str::to_string).collect())
            .unwrap_or_default(),
    };
    let mut missing = Vec::new();
    for (k, s) in [
        ("name", &a.name),
        ("purpose", &a.purpose),
        ("audience", &a.audience),
        ("owner", &a.owner),
        ("approach", &a.approach),
    ] {
        if s.is_empty() {
            missing.push(k)
        }
    }
    (a, missing)
}

#[derive(Debug, Default)]
pub struct Started {
    pub packs: Vec<String>,
    pub added: usize,
}

/// Set a project up from the answers: add the core bundle and the chosen packs, and write the overview. Refused when the
/// project already has an overview (change that with an ordinary edit instead). `offered` says which pack names may be added.
pub fn start(model: &Path, a: &Answers, all: &[Pack], offered: &dyn Fn(&str) -> bool) -> Result<Started, String> {
    if a.name.is_empty() || a.purpose.is_empty() || a.audience.is_empty() || a.owner.is_empty() {
        return Err("name, purpose, audience and owner are all needed".into());
    }
    for s in [&a.name, &a.purpose, &a.audience, &a.owner] {
        if s.chars().count() > LIMIT {
            return Err(format!("keep each answer under {LIMIT} characters"));
        }
    }
    if !APPROACHES.contains(&a.approach.as_str()) {
        return Err(format!("approach must be one of {}", APPROACHES.join(", ")));
    }
    for p in &a.packs {
        if !offered(p) || all.iter().all(|x| &x.name != p || x.role == "core") {
            return Err(format!("{p} is not one of the packs on offer"));
        }
    }
    let files = crate::read_model(&model.to_string_lossy())?;
    if files.iter().filter_map(|(_, t)| parse_node(t).ok()).any(|n| n.id == "overview") {
        return Err("this project already has an overview: change it with put_node, or add packs with add_pack".into());
    }
    let mut out = Started::default();
    let have: Vec<String> = installed(model)?.into_iter().map(|(n, _)| n).collect();
    for name in std::iter::once("core").chain(a.packs.iter().map(String::as_str)) {
        if have.iter().any(|n| n == name) {
            continue;
        }
        for (n, added) in add_tree_with(all, name, None, model, false, Clash::Skip)? {
            out.added += added.added.len();
            out.packs.push(n);
        }
    }
    let plane = crate::read_model(&model.to_string_lossy())?
        .into_iter()
        .filter_map(|(_, t)| parse_node(&t).ok())
        .find(|n| n.kind.as_deref() == Some("plane") && (n.id == "about" || n.id.ends_with("--about")))
        .map(|n| n.id);
    let mut node = Node {
        id: "overview".into(),
        kind: Some("overview".into()),
        title: a.name.clone(),
        parent: None,
        plane,
        fields: Default::default(),
        links: Vec::new(),
        body: String::new(),
    };
    for (k, v) in [
        ("purpose", &a.purpose),
        ("audience", &a.audience),
        ("owner", &a.owner),
        ("stage", &"design".to_string()),
        ("approach", &a.approach),
    ] {
        node.fields.insert(k.to_string(), Value::String(v.clone()));
    }
    fs::write(model.join("overview.md"), node_to_markdown(&node))
        .map_err(|e| format!("cannot write overview.md: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("trumly-start-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn answers() -> Answers {
        Answers {
            name: "Client portal".into(),
            purpose: "A portal for clients".into(),
            audience: "The team".into(),
            owner: "Platform".into(),
            approach: "greenfield".into(),
            packs: vec![],
        }
    }

    #[test]
    fn the_questions_are_the_ones_the_spec_lists() {
        let q = questions(&[("api".into(), "HTTP interfaces".into())]);
        let ids: Vec<&str> = q["questions"].as_array().unwrap().iter().map(|x| x["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["name", "kind", "purpose", "audience", "owner", "approach", "packs"]);
        assert_eq!(q["questions"][6]["options"][0]["value"], "api");
    }

    #[test]
    fn answers_missing_a_required_one_say_which() {
        let (_, missing) = read_answers(&json!({"name": "X", "approach": "brownfield"}));
        assert_eq!(missing, ["purpose", "audience", "owner"]);
    }

    #[test]
    fn a_new_project_gets_the_core_and_an_overview_that_passes_its_rule() {
        let dir = tmp("ok");
        let lib = crate::library::load(None).unwrap();
        let all = lib.packs();
        let done = start(&dir, &answers(), &all, &|_| false).unwrap();
        assert!(done.packs.contains(&"core".to_string()) && done.added > 5);
        let nodes: Vec<Node> = crate::read_model(&dir.to_string_lossy())
            .unwrap()
            .into_iter()
            .filter_map(|(_, t)| parse_node(&t).ok())
            .collect();
        let ov = nodes.iter().find(|n| n.id == "overview").expect("overview");
        assert_eq!(ov.kind.as_deref(), Some("overview"));
        assert_eq!(ov.fields["stage"], "design");
        assert_eq!(ov.fields["approach"], "greenfield");
        assert!(ov.plane.as_ref().is_some_and(|p| nodes.iter().any(|n| &n.id == p)), "{:?}", ov.plane);
        // a second start is refused, and nothing else is touched
        assert!(start(&dir, &answers(), &all, &|_| false).unwrap_err().contains("already has an overview"));
    }

    #[test]
    fn it_refuses_an_approach_it_does_not_know_and_a_pack_not_on_offer() {
        let dir = tmp("bad");
        let lib = crate::library::load(None).unwrap();
        let all = lib.packs();
        assert!(start(&dir, &Answers { approach: "ship".into(), ..answers() }, &all, &|_| true).is_err());
        assert!(
            start(&dir, &Answers { packs: vec!["nope".into()], ..answers() }, &all, &|_| true)
                .unwrap_err()
                .contains("not one of the packs")
        );
        assert!(fs::read_dir(&dir).unwrap().next().is_none(), "nothing was written");
    }
}
