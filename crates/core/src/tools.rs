//! The read side of the agent tools, as pure functions over a checked project. The local MCP server, the hosted
//! one and the browser all call these, so the same question gets the same answer everywhere.
//!
//! What stays with the host: the revision and path of a file (they depend on where the files live), `status`
//! (it depends on git or on the cloud's last sync), and actually writing changes.

use serde_json::{Value, json};

use crate::model::Node;
use crate::ops::rename_node;
use crate::report::{Report, node_to_json};
use crate::vocab;

/// Run the tool `name` over `report`. `None` means this module has no such tool, so the host may offer its own.
///
/// Besides the agent-facing tools there are two the hosts use to build their write tools:
/// `_standing` (how the project stands after a change, with the findings about `id`) and
/// `_rename_edit` (what renaming `from` to `to` would write and delete, as nodes) and
/// `_prepare` (a node about to be written: the node with what the vocabulary knows filled in, what was, and advice).
/// `get_index` takes a host-set `_write` flag, so a read-only agent is not asked to write an overview.
pub fn call(report: &Report, name: &str, args: &Value) -> Option<Result<Value, String>> {
    Some(match name {
        "check" => Ok(check_summary(report)),
        "get_index" => Ok(json!(vocab::agent_index(report, args["_write"].as_bool().unwrap_or(false)))),
        "describe_types" => Ok(vocab::describe_types(report, args)),
        "_prepare" => Ok(vocab::prepare(report, &args["node"], args["is_new"].as_bool().unwrap_or(true))),
        "get_node" => get_node(report, args),
        "list_nodes" => Ok(list_nodes(report, args)),
        "search" => search(report, args),
        "get_links" => get_links(report, args),
        "_standing" => Ok(standing(report, args["id"].as_str())),
        "_rename_edit" => rename_edit(report, args),
        _ => return None,
    })
}

fn failed_rules(report: &Report) -> Vec<Value> {
    report.to_json()["results"].as_array().into_iter().flatten().filter(|r| r["status"] == "fail").cloned().collect()
}

fn check_summary(report: &Report) -> Value {
    let j = report.to_json();
    json!({
        "nodes": report.graph.len(),
        "load_errors": j["load_errors"],
        "structure": j["structure"],
        "failed_rules": failed_rules(report),
        "ghosts": j["ghosts"],
        "ok": report.exit_code() == 0,
    })
}

/// How the project stands after a change: whole-model totals, and, given a node id, what the checks say about that node.
pub fn standing(report: &Report, id: Option<&str>) -> Value {
    let j = report.to_json();
    let failed = failed_rules(report);
    let about = |list: &Value| -> Vec<Value> {
        list.as_array()
            .map(|a| a.iter().filter(|f| id.is_some_and(|i| f["node"] == i || f["ref"] == i)).cloned().collect())
            .unwrap_or_default()
    };
    let mut out = json!({
        "model": {"ok": report.exit_code() == 0, "errors": j["structure"]["errors"].as_array().map_or(0, Vec::len), "failed_rules": failed.len()},
    });
    if id.is_some() {
        out["findings"] = json!({
            "errors": about(&j["structure"]["errors"]),
            "warnings": about(&j["structure"]["warnings"]),
            "failed_rules": failed.iter().filter(|r| id.is_some_and(|i| r["node"] == i)).collect::<Vec<_>>(),
        });
    }
    out
}

fn get_node(report: &Report, args: &Value) -> Result<Value, String> {
    let id = args["id"].as_str().ok_or("get_node needs an 'id'")?;
    let node = report.graph.get(id).ok_or(format!("no node with id '{id}'"))?;
    let inbound: Vec<Value> = report
        .graph
        .nodes()
        .flat_map(|o| o.links.iter().filter(|l| l.to == id).map(move |l| json!({"from": o.id, "type": l.kind})))
        .collect();
    let rules: Vec<Value> = report
        .evaluation
        .results
        .iter()
        .filter(|r| r.node == id)
        .map(|r| json!({"rule": r.rule, "severity": r.severity, "status": r.status.as_str()}))
        .collect();
    let mut out = json!({"node": node_to_json(node), "inbound": inbound, "rules": rules});
    if let Some(row) = report.derived.get(id) {
        out["derived"] = crate::derive::row_to_json(row);
    }
    Ok(out)
}

fn is_under<'a>(report: &'a Report, mut id: &'a str, ancestor: &str) -> bool {
    let mut hops = 0;
    while let Some(parent) = report.graph.get(id).and_then(|n| n.parent.as_deref()) {
        if parent == ancestor {
            return true;
        }
        id = parent;
        hops += 1;
        if hops > report.graph.len() {
            break; // a parent cycle; stop rather than loop forever
        }
    }
    false
}

fn list_nodes(report: &Report, args: &Value) -> Value {
    let want = |k: &str| args[k].as_str();
    let rows: Vec<Value> = report
        .graph
        .nodes()
        .filter(|n| want("type").is_none_or(|t| n.kind.as_deref() == Some(t)))
        .filter(|n| want("plane").is_none_or(|p| n.plane.as_deref() == Some(p)))
        .filter(|n| want("under").is_none_or(|a| is_under(report, &n.id, a)))
        .map(|n| json!({"id": n.id, "type": n.kind, "title": n.title, "plane": n.plane, "parent": n.parent}))
        .collect();
    json!(rows)
}

fn contains_ci(hay: &str, needle_lc: &str) -> Option<usize> {
    let lc = hay.to_lowercase();
    lc.find(needle_lc).map(|b| lc[..b].chars().count())
}

fn snippet(text: &str, at: usize, len: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let from = at.saturating_sub(40).min(chars.len());
    let to = (at + len + 60).min(chars.len());
    let mut s: String = chars[from..to].iter().collect::<String>().replace('\n', " ");
    if from > 0 {
        s.insert(0, '…');
    }
    if to < chars.len() {
        s.push('…');
    }
    s
}

fn search(report: &Report, args: &Value) -> Result<Value, String> {
    let query = args["query"].as_str().ok_or("search needs a 'query'")?.trim().to_lowercase();
    if query.is_empty() {
        return Err("search needs a non-empty 'query'".into());
    }
    let limit = args["limit"].as_u64().unwrap_or(20).clamp(1, 100) as usize;
    let mut rows = Vec::new();
    let mut total = 0;
    for n in report.graph.nodes().filter(|n| args["type"].as_str().is_none_or(|t| n.kind.as_deref() == Some(t))) {
        let mut matched = Vec::new();
        let mut snip = None;
        if n.id.to_lowercase().contains(&query) {
            matched.push("id");
        }
        if n.title.to_lowercase().contains(&query) {
            matched.push("title");
        }
        let fields = serde_json::to_string(&n.fields).unwrap_or_default();
        if let Some(at) = contains_ci(&fields, &query) {
            matched.push("fields");
            snip.get_or_insert_with(|| snippet(&fields, at, query.chars().count()));
        }
        // derived values (a calculated name) are searched too, so a real name found in a console or an alert finds its node
        let derived = report
            .derived
            .get(&n.id)
            .map(|row| {
                row.values()
                    .filter_map(|d| match d {
                        crate::derive::Derived::Values(v) => Some(v.join("\n")),
                        crate::derive::Derived::Overridden { values, .. } => Some(values.join("\n")),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        if let Some(at) = contains_ci(&derived, &query) {
            matched.push("derived");
            snip.get_or_insert_with(|| snippet(&derived, at, query.chars().count()));
        }
        if let Some(at) = contains_ci(&n.body, &query) {
            matched.push("body");
            snip.get_or_insert_with(|| snippet(&n.body, at, query.chars().count()));
        }
        if matched.is_empty() {
            continue;
        }
        total += 1;
        if rows.len() < limit {
            rows.push(json!({"id": n.id, "type": n.kind, "title": n.title, "matched": matched, "snippet": snip}));
        }
    }
    Ok(json!({"total": total, "shown": rows.len(), "results": rows}))
}

fn get_links(report: &Report, args: &Value) -> Result<Value, String> {
    let id = args["id"].as_str().ok_or("get_links needs an 'id'")?;
    let node = report.graph.get(id).ok_or(format!("no node with id '{id}'"))?;
    let outbound: Vec<Value> = node
        .links
        .iter()
        .map(|l| {
            let target = report.graph.get(&l.to);
            json!({"type": l.kind, "to": l.to, "key": l.key, "fields": l.fields, "to_title": target.map(|t| t.title.clone()), "exists": target.is_some()})
        })
        .collect();
    let inbound: Vec<Value> = report
        .graph
        .nodes()
        .flat_map(|o| {
            o.links
                .iter()
                .filter(|l| l.to == id)
                .map(move |l| json!({"from": o.id, "from_title": o.title, "type": l.kind, "key": l.key}))
        })
        .collect();
    Ok(json!({"id": id, "outbound": outbound, "inbound": inbound}))
}

fn rename_edit(report: &Report, args: &Value) -> Result<Value, String> {
    let from = args["from"].as_str().ok_or("rename_node needs 'from'")?;
    let to = args["to"].as_str().ok_or("rename_node needs 'to'")?;
    let edit = rename_node(&report.graph, from, to)?;
    let nodes: Vec<Value> = edit.write.iter().map(node_to_json).collect();
    Ok(json!({"write": nodes, "delete": edit.delete, "renamed": edit.renamed}))
}

/// Dangling links left if node `id` is deleted: who points at it.
pub fn inbound_links(report: &Report, id: &str) -> Vec<Value> {
    report
        .graph
        .nodes()
        .filter(|o| o.id != id)
        .flat_map(|o: &Node| o.links.iter().filter(|l| l.to == id).map(move |l| json!({"from": o.id, "type": l.kind})))
        .collect()
}
