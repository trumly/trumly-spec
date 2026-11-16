//! A tiny collector for the usage events the editor and CLI send (see `trumly_fs::telemetry`).
//!
//! `POST /v1/events` takes `{"events": [...]}` and appends the valid ones to a JSON-lines file; `GET /v1/summary`
//! counts them. It keeps nothing but what the events carry (no IP address, no headers). Run it anywhere:
//! `trumly collector --port 4748 --file events.jsonl`, and point `TRUMLY_TELEMETRY_URL` at `http://host:4748/v1/events`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde_json::{Value, json};
use tiny_http::{Header, Method, Request, Response, Server};

const MAX_BODY: u64 = 256 * 1024;
const MAX_EVENTS: usize = 200;

fn header(k: &str, v: &str) -> Header {
    Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap()
}

fn reply(req: Request, status: u16, body: Value) {
    let r = Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("Access-Control-Allow-Origin", "*"))
        .with_header(header("Access-Control-Allow-Methods", "POST, GET, OPTIONS"))
        .with_header(header("Access-Control-Allow-Headers", "Content-Type"));
    let _ = req.respond(r);
}

/// The shape every accepted event must have; anything else is dropped, so the file holds nothing unexpected.
pub fn valid(e: &Value) -> Option<Value> {
    let o = e.as_object()?;
    let name = o.get("event")?.as_str()?;
    let ok_name = !name.is_empty()
        && name.len() <= 48
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    let short = |k: &str| o.get(k).and_then(Value::as_str).filter(|s| s.len() <= 64).map(str::to_string);
    let (ts, install, version, os, src) =
        (short("ts")?, short("install")?, short("version")?, short("os")?, short("src")?);
    let props = match o.get("props") {
        Some(Value::Object(m))
            if m.len() <= 12
                && m.iter().all(|(k, v)| {
                    k.len() <= 32 && (v.is_boolean() || v.is_number() || v.as_str().is_some_and(|s| s.len() <= 48))
                }) =>
        {
            Value::Object(m.clone())
        }
        None => json!({}),
        _ => return None,
    };
    ok_name.then(|| json!({"event": name, "ts": ts, "install": install, "version": version, "os": os, "src": src, "props": props}))
}

fn summary(file: &PathBuf) -> Value {
    let text = fs::read_to_string(file).unwrap_or_default();
    let mut events: BTreeMap<String, usize> = BTreeMap::new();
    let mut installs = BTreeSet::new();
    let mut days: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut total = 0;
    for l in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(l) else { continue };
        total += 1;
        let (name, inst) = (v["event"].as_str().unwrap_or(""), v["install"].as_str().unwrap_or(""));
        *events.entry(name.to_string()).or_default() += 1;
        installs.insert(inst.to_string());
        if let Some(d) = v["ts"].as_str().and_then(|t| t.get(..10)) {
            days.entry(d.to_string()).or_default().insert(inst.to_string());
        }
        versions.entry(v["version"].as_str().unwrap_or("").to_string()).or_default().insert(inst.to_string());
    }
    json!({
        "events": total, "installs": installs.len(), "by_event": events,
        "active_installs_by_day": days.iter().map(|(d, s)| (d.clone(), s.len())).collect::<BTreeMap<_, _>>(),
        "installs_by_version": versions.iter().map(|(d, s)| (d.clone(), s.len())).collect::<BTreeMap<_, _>>(),
    })
}

pub fn run(addr: &str, file: PathBuf) -> Result<(), String> {
    let server = Server::http(addr).map_err(|e| format!("cannot listen on {addr}: {e}"))?;
    eprintln!("trumly collector on http://{addr}  (events go to {})", file.display());
    for mut req in server.incoming_requests() {
        let path = req.url().split('?').next().unwrap_or("").to_string();
        match (req.method().clone(), path.as_str()) {
            (Method::Options, _) => reply(req, 204, json!({})),
            (Method::Post, "/v1/events") => {
                let mut body = String::new();
                if req.as_reader().take(MAX_BODY).read_to_string(&mut body).is_err() {
                    reply(req, 400, json!({"error": "unreadable body"}));
                    continue;
                }
                let Some(list) = serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| v.get("events").and_then(Value::as_array).cloned())
                else {
                    reply(req, 400, json!({"error": "expected {\"events\": [...]}"}));
                    continue;
                };
                let good: Vec<Value> = list.iter().take(MAX_EVENTS).filter_map(valid).collect();
                if !good.is_empty() {
                    if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
                        let _ = fs::create_dir_all(dir);
                    }
                    match OpenOptions::new().create(true).append(true).open(&file) {
                        Ok(mut f) => {
                            for e in &good {
                                let _ = writeln!(f, "{e}");
                            }
                        }
                        Err(e) => {
                            reply(req, 500, json!({"error": format!("cannot write: {e}")}));
                            continue;
                        }
                    }
                }
                reply(req, 200, json!({"accepted": good.len(), "dropped": list.len().min(MAX_EVENTS) - good.len()}));
            }
            (Method::Get, "/v1/summary") => reply(req, 200, summary(&file)),
            (Method::Get, "/v1/health") => reply(req, 200, json!({"ok": true})),
            _ => reply(req, 404, json!({"error": "not found"})),
        }
    }
    Ok(())
}
