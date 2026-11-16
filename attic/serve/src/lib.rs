//! `trumly serve`: a local HTTP API over a project folder, and (optionally) a page that uses it.
//!
//! The files stay the source of truth. The server parses them, hands nodes to the editor as JSON,
//! writes edited nodes back as canonical Markdown, and tells the editor when files change on disk.
//!
//! Safety, because a server that writes files on localhost is a target for any web page you visit:
//!   * it listens on 127.0.0.1 only;
//!   * every /api request needs the startup token, which only the page we serve ever receives;
//!   * the Host header must be localhost, so a DNS-rebinding page is refused;
//!   * file paths are confined to the project folder and to `.md` files.

pub mod collector;

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use serde_json::{Value, json};
use tiny_http::{Header, Method, Request, Response, Server};
use trumly_core::format::node_to_markdown;
use trumly_core::model::Node;
use trumly_core::parse::{node_from_map, parse_node};
use trumly_core::report::{check, node_to_json};

pub struct Options {
    pub root: PathBuf,
    pub addr: String,            // e.g. "127.0.0.1:4747"; port 0 picks a free one
    pub page: Option<PathBuf>,   // an HTML file to serve at "/" (the editor)
    pub token: Option<String>,   // fixed token (for tests); otherwise random
    pub packs: Option<PathBuf>,  // a folder of packs the editor may add to the project
    pub state: Option<PathBuf>,  // where the list of recent projects is kept; ~/.trumly when unset
    pub browse: Option<PathBuf>, // the folder the editor may look in and open projects from; your home folder when unset
}

pub struct Running {
    pub addr: std::net::SocketAddr,
    pub token: String,
}

/// What the server knows about the folder right now. Behind a mutex; `changed` wakes long-polls.
struct Shared {
    files: BTreeMap<String, String>, // path -> rev, for files that exist
    gone: BTreeMap<String, u64>,     // path -> version at which it disappeared
    touched: BTreeMap<String, u64>,  // path -> version at which it last changed
    version: u64,
}

struct App {
    /// the project folder; the editor can switch it, so read it with `root()`
    root: RwLock<PathBuf>,
    browse: PathBuf,
    recent: Option<PathBuf>, // the file listing recently opened projects
    page: Option<PathBuf>,
    packs: Option<PathBuf>,
    token: String,
    port: u16,
    state: Mutex<Shared>,
    changed: Condvar,
}

// ---------- folder scanning ----------

fn rev_of(path: &Path) -> Option<String> {
    let m = fs::metadata(path).ok()?;
    let t = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    Some(format!("{t}-{}", m.len()))
}

fn scan(root: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(root, &p, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                if let (Ok(rel), Some(rev)) = (p.strip_prefix(root), rev_of(&p)) {
                    out.insert(rel.to_string_lossy().replace('\\', "/"), rev);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

impl App {
    /// Compare the folder with what we last saw and record the differences.
    fn root(&self) -> PathBuf {
        self.root.read().unwrap().clone()
    }

    fn refresh(&self) {
        let root = self.root.read().unwrap(); // held until we are done, so a folder switch cannot land in the middle
        let now = scan(&root);
        let mut s = self.state.lock().unwrap();
        let mut bumped = false;
        let old = s.files.clone();
        for (p, rev) in &now {
            if old.get(p) != Some(rev) {
                if !bumped {
                    s.version += 1;
                    bumped = true;
                }
                let v = s.version;
                s.touched.insert(p.clone(), v);
                s.gone.remove(p);
            }
        }
        for p in old.keys().filter(|p| !now.contains_key(*p)) {
            if !bumped {
                s.version += 1;
                bumped = true;
            }
            let v = s.version;
            s.gone.insert(p.clone(), v);
            s.touched.remove(p);
        }
        s.files = now;
        if bumped {
            self.changed.notify_all();
        }
    }

    /// Record a write we just made ourselves, so the scanner does not report it back as an outside change.
    fn note_write(&self, rel: &str) -> String {
        let rev = rev_of(&self.root().join(rel)).unwrap_or_default();
        let mut s = self.state.lock().unwrap();
        s.version += 1;
        let v = s.version;
        s.files.insert(rel.to_string(), rev.clone());
        s.touched.insert(rel.to_string(), v);
        s.gone.remove(rel);
        self.changed.notify_all();
        rev
    }

    fn note_delete(&self, rel: &str) {
        let mut s = self.state.lock().unwrap();
        s.version += 1;
        let v = s.version;
        s.files.remove(rel);
        s.touched.remove(rel);
        s.gone.insert(rel.to_string(), v);
        self.changed.notify_all();
    }

    fn path_for_id(&self, id: &str) -> Option<String> {
        let s = self.state.lock().unwrap();
        s.files.keys().find(|p| stem(p) == id).cloned()
    }

    fn rev_now(&self, rel: &str) -> Option<String> {
        rev_of(&self.root().join(rel))
    }
}

fn stem(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".md").unwrap_or(name)
}

// ---------- JSON shapes ----------

fn node_with_file(n: &Node, path: &str, rev: &str) -> Value {
    let mut v = node_to_json(n);
    v["path"] = json!(path);
    v["rev"] = json!(rev);
    v
}

fn read_node(app: &App, rel: &str) -> Option<(Node, String)> {
    let text = fs::read_to_string(app.root().join(rel)).ok()?;
    let rev = app.rev_now(rel)?;
    parse_node(&text).ok().map(|n| (n, rev))
}

// ---------- HTTP plumbing ----------

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn reply(req: Request, status: u16, body: Value) {
    let r = Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("Cache-Control", "no-store"));
    let _ = req.respond(r);
}

fn find_header(req: &Request, name: &str) -> Option<String> {
    req.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.to_string())
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| kv.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| percent_decode(v)))
}

/// Ids become file names, so they must be plain.
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && !id.starts_with('.')
        && !id.contains(['/', '\\', '\0'])
        && id.chars().all(|c| !c.is_control())
}

/// The stricter rule for ids of files we create. A name that works on every system and cannot be
/// mistaken for another: lower-case letters, digits, `-` and `_`; starts and ends with a letter or digit;
/// at most 100 characters; not a name Windows reserves (`con`, `nul`, `com1`...); and no `--`, which only a pack's nodes use as a namespace separator. Files that already
/// exist keep whatever name they have, so older projects still open and save.
fn portable_id(id: &str, namespaced: bool) -> bool {
    trumly_fs::portable_id(id, namespaced)
}

/// Where view state is kept, relative to the project folder.
const VIEW_FILE: &str = ".trumly/view.json";
const MAX_VIEW_BYTES: usize = 1 << 20;

fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    trumly_fs::write_atomically(path, text)
}

fn handle(app: &Arc<App>, mut req: Request) {
    // 1. Host check: only localhost names on our port (blocks DNS rebinding).
    let host = find_header(&req, "Host").unwrap_or_default();
    if host != format!("127.0.0.1:{}", app.port) && host != format!("localhost:{}", app.port) {
        return reply(req, 403, json!({"error": "bad host"}));
    }

    let url = req.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let method = req.method().clone();

    // 2. The page.
    if method == Method::Get && (path == "/" || path == "/index.html") {
        let html = match &app.page {
            Some(p) => fs::read_to_string(p).unwrap_or_else(|e| format!("cannot read {}: {e}", p.display())),
            None => format!(
                "<!doctype html><meta charset=utf-8><title>trumly</title><body style=\"font:15px system-ui;margin:2rem\"><h1>trumly serve</h1><p>Serving <code>{}</code>.</p><p>The API is running. Start with an editor page: <code>trumly serve DIR --page editor.html</code></p>",
                app.root().display()
            ),
        };
        let page = format!("<meta name=\"trumly-token\" content=\"{}\">{html}", app.token);
        let r = Response::from_string(page)
            .with_header(header("Content-Type", "text/html; charset=utf-8"))
            .with_header(header("Cache-Control", "no-store"));
        return {
            let _ = req.respond(r);
        };
    }

    // 3. Everything else is the API and needs the token.
    if !path.starts_with("/api/") {
        return reply(req, 404, json!({"error": "not found"}));
    }
    if !find_header(&req, "X-Trumly-Token").is_some_and(|t| same_token(&t, &app.token)) {
        return reply(req, 403, json!({"error": "missing or wrong token"}));
    }

    let mut body = String::new();
    if method == Method::Put || method == Method::Post {
        if req.as_reader().take(5_000_000).read_to_string(&mut body).is_err() {
            return reply(req, 400, json!({"error": "unreadable body"}));
        }
    }

    match (method, path) {
        (Method::Get, "/api/info") => {
            let root = app.root();
            let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let version = app.state.lock().unwrap().version;
            reply(req, 200, json!({"name": name, "root": root.display().to_string(), "version": version}))
        }
        (Method::Get, "/api/dirs") => dirs_list(app, req, query_param(query, "path")),
        (Method::Get, "/api/recent") => recent_list(app, req),
        (Method::Post, "/api/open") => open_folder(app, req, &body),
        (Method::Get, "/api/model") => {
            app.refresh();
            let files = trumly_fs::read_model(&app.root().to_string_lossy()).unwrap_or_default();
            let report = check(files);
            let (version, revs) = {
                let s = app.state.lock().unwrap();
                (s.version, s.files.clone())
            };
            let nodes: Vec<Value> = report
                .graph
                .nodes()
                .map(|n| {
                    let p = revs.keys().find(|p| stem(p) == n.id).cloned().unwrap_or_default();
                    node_with_file(n, &p, revs.get(&p).map(String::as_str).unwrap_or(""))
                })
                .collect();
            let j = report.to_json();
            reply(req, 200, json!({"version": version, "nodes": nodes, "load_errors": j["load_errors"]}))
        }
        (Method::Get, "/api/check") => {
            app.refresh();
            let files = trumly_fs::read_model(&app.root().to_string_lossy()).unwrap_or_default();
            reply(req, 200, check(files).to_json())
        }
        (Method::Get, "/api/packs") => packs_list(app, req),
        (Method::Get, p) if p.starts_with("/api/packs/") && (p.contains("/examples/") || p.contains("/patterns/")) => {
            // /api/packs/<pack>/examples/<name> or /patterns/<name>: one in full
            let rest = &p["/api/packs/".len()..];
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() != 3 {
                return reply(req, 404, json!({"error": "not found"}));
            }
            let (pack, kind, name) = (
                percent_decode(parts[0]),
                if parts[1] == "examples" { "example" } else { "pattern" },
                percent_decode(parts[2]),
            );
            match trumly_fs::library::load(app.packs.as_deref()) {
                Err(e) => reply(req, 500, json!({"error": e})),
                Ok(lib) => {
                    match lib.entries.iter().find(|e| e.pack.name == pack).and_then(|e| e.pack.sample(kind, &name)) {
                        Some(s) => reply(req, 200, {
                            let mut v = s.full();
                            v["pack"] = json!(pack);
                            v
                        }),
                        None => reply(req, 404, json!({"error": format!("{pack} has no {kind} called {name}")})),
                    }
                }
            }
        }
        (Method::Get, "/api/packs/status") => match trumly_fs::pack::status(&app.root()) {
            Ok(list) => reply(
                req,
                200,
                json!({"packs": list.iter().map(|s| json!({"name": s.name, "version": s.version, "clean": s.clean, "changed": s.changed, "missing": s.missing})).collect::<Vec<_>>()}),
            ),
            Err(e) => reply(req, 500, json!({"error": e})),
        },
        (Method::Delete, p) if p.starts_with("/api/packs/") => {
            let name = percent_decode(&p["/api/packs/".len()..]);
            match trumly_fs::pack::remove(&name, &app.root(), query_param(query, "force").as_deref() == Some("1")) {
                Ok(r) => {
                    app.refresh();
                    reply(req, 200, json!({"removed": r.removed}))
                }
                Err(e) => reply(req, 409, json!({"error": e})),
            }
        }
        (Method::Post, p) if p.starts_with("/api/packs/") && p.ends_with("/detach") => {
            let name = percent_decode(&p["/api/packs/".len()..p.len() - "/detach".len()]);
            match trumly_fs::pack::detach(&name, &app.root(), query_param(query, "force").as_deref() == Some("1")) {
                Ok(r) => {
                    app.refresh();
                    reply(req, 200, json!({"kept": r.kept}))
                }
                Err(e) => reply(req, 409, json!({"error": e})),
            }
        }
        (Method::Post, p) if p.starts_with("/api/packs/") && p.ends_with("/export") => {
            let name = percent_decode(&p["/api/packs/".len()..p.len() - "/export".len()]);
            let lib = match trumly_fs::library::load(app.packs.as_deref()) {
                Ok(l) => l,
                Err(e) => return reply(req, 500, json!({"error": e})),
            };
            match trumly_fs::library::export(&app.root(), &lib, &name) {
                Ok((n, dir)) => reply(req, 200, json!({"exported": n, "dir": dir.display().to_string()})),
                Err(e) => reply(req, 409, json!({"error": e})),
            }
        }
        (Method::Post, p) if p.starts_with("/api/packs/") => {
            let name = percent_decode(&p["/api/packs/".len()..]);
            let only = query_param(query, "only")
                .map(|o| o.split(',').filter(|x| !x.is_empty()).map(str::to_string).collect::<Vec<_>>());
            let clash = match query_param(query, "clash").as_deref() {
                Some("skip") => trumly_fs::pack::Clash::Skip,
                Some("replace") => trumly_fs::pack::Clash::Replace,
                _ => trumly_fs::pack::Clash::Refuse,
            };
            packs_add(app, req, &name, only, query_param(query, "force").as_deref() == Some("1"), clash)
        }
        (Method::Get, "/api/telemetry") => {
            use trumly_fs::telemetry as t;
            let sample = json!({"event": "pack_added", "ts": t::now_iso(), "install": "<random id, made when you turn this on>", "version": env!("CARGO_PKG_VERSION"), "os": std::env::consts::OS, "src": "editor", "props": {"pack": "system-design", "nodes": "10-49"}});
            reply(
                req,
                200,
                json!({"state": t::state().as_str(), "chosen": t::chosen().as_str(), "forced_off": t::forced_off(), "endpoint": t::endpoint(), "queued": t::queued_count(), "sample": sample}),
            )
        }
        (Method::Put, "/api/telemetry") => {
            use trumly_fs::telemetry as t;
            let want = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| v.get("state").and_then(Value::as_str).and_then(t::State::parse));
            match want {
                Some(s) if s != t::State::Unset => match t::set_state(s) {
                    Ok(()) => reply(req, 200, json!({"state": t::state().as_str()})),
                    Err(e) => reply(req, 500, json!({"error": e})),
                },
                _ => reply(req, 400, json!({"error": "state must be \"on\" or \"off\""})),
            }
        }
        (Method::Get, "/api/telemetry/queue") => {
            let max = query_param(query, "max").and_then(|m| m.parse().ok()).unwrap_or(100usize).min(500);
            reply(req, 200, json!({"events": trumly_fs::telemetry::queued(max)}))
        }
        (Method::Post, "/api/telemetry/ack") => {
            let n = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| v.get("count").and_then(Value::as_u64))
                .unwrap_or(0) as usize;
            trumly_fs::telemetry::ack(n);
            reply(req, 200, json!({"queued": trumly_fs::telemetry::queued_count()}))
        }
        (Method::Post, "/api/telemetry/event") => {
            let v = serde_json::from_str::<Value>(&body).unwrap_or(Value::Null);
            let name = v.get("event").and_then(Value::as_str).unwrap_or("");
            let ok = trumly_fs::telemetry::record(name, v.get("props").cloned().unwrap_or(Value::Null));
            reply(req, 200, json!({"recorded": ok}))
        }
        (Method::Get, "/api/changes") => {
            let since: u64 = query_param(query, "since").and_then(|v| v.parse().ok()).unwrap_or(0);
            // Long poll: wait up to 20s for something newer than `since`.
            let deadline = std::time::Instant::now() + Duration::from_secs(20);
            let mut s = app.state.lock().unwrap();
            while s.version <= since {
                let left = deadline.saturating_duration_since(std::time::Instant::now());
                if left.is_zero() {
                    break;
                }
                s = app.changed.wait_timeout(s, left).unwrap().0;
            }
            let version = s.version;
            let touched: Vec<String> = s.touched.iter().filter(|(_, v)| **v > since).map(|(p, _)| p.clone()).collect();
            let removed: Vec<String> =
                s.gone.iter().filter(|(_, v)| **v > since).map(|(p, _)| stem(p).to_string()).collect();
            drop(s);
            let mut nodes = Vec::new();
            let mut problems = Vec::new();
            for p in touched {
                match read_node(app, &p) {
                    Some((n, rev)) if n.id == stem(&p) => nodes.push(node_with_file(&n, &p, &rev)),
                    _ => problems.push(p),
                }
            }
            reply(req, 200, json!({"version": version, "nodes": nodes, "removed": removed, "problems": problems}))
        }
        // View state (pinned positions, saved camera views) lives in the folder, beside the project, so it travels with git.
        (Method::Get, "/api/view") => {
            let text = fs::read_to_string(app.root().join(VIEW_FILE)).unwrap_or_default();
            let view: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({}));
            reply(req, 200, json!({"view": view, "exists": !text.is_empty()}))
        }
        (Method::Put, "/api/view") => match serde_json::from_str::<Value>(&body) {
            Ok(v) if v.is_object() && body.len() <= MAX_VIEW_BYTES => {
                let path = app.root().join(VIEW_FILE);
                let text = serde_json::to_string_pretty(&v).unwrap_or_default() + "\n";
                let r = path.parent().map(fs::create_dir_all).transpose().and_then(|_| write_atomically(&path, &text));
                match r {
                    Ok(()) => reply(req, 200, json!({"ok": true})),
                    Err(e) => reply(req, 500, json!({"error": format!("write failed: {e}")})),
                }
            }
            _ => reply(req, 400, json!({"error": "view must be a JSON object under 1 MB"})),
        },
        (Method::Put, p) if p.starts_with("/api/nodes/") => {
            let id = percent_decode(&p["/api/nodes/".len()..]);
            put_node(app, req, &id, &body)
        }
        (Method::Delete, p) if p.starts_with("/api/nodes/") => {
            let id = percent_decode(&p["/api/nodes/".len()..]);
            delete_node(app, req, &id, query_param(query, "base_rev"))
        }
        _ => reply(req, 404, json!({"error": "not found"})),
    }
}

/// The packs on offer, with what the project already has. No packs folder is an empty list, not an error.
fn packs_list(app: &Arc<App>, req: Request) {
    let lib = match trumly_fs::library::load(app.packs.as_deref()) {
        Ok(l) => l,
        Err(e) => return reply(req, 500, json!({"error": e})),
    };
    let have = trumly_fs::pack::installed(&app.root()).unwrap_or_default();
    let all = lib.packs();
    let problems = trumly_fs::pack::lint(&all);
    let packs: Vec<Value> = lib.entries.iter().map(|e| {
        let p = &e.pack;
        json!({
            "name": p.name, "title": p.title, "version": p.version, "description": p.description, "namespace": p.namespace, "license": p.license, "publisher": p.publisher, "role": p.role, "use_when": p.use_when, "enriches": p.enriches, "requires": p.requires, "includes": p.includes,
            "nodes": p.files.len().saturating_sub(1), "source": e.source, "contents": p.contents(), "samples": p.samples_summary(),
            "problems": problems.iter().filter(|x| x.pack == p.name).map(|x| json!({"severity": x.severity, "message": x.message})).collect::<Vec<_>>(),
            "installed": have.iter().find(|(n, _)| *n == p.name).map(|(_, v)| v.clone()),
        })
    }).collect();
    reply(req, 200, json!({"dir": lib.export_dir.display().to_string(), "packs": packs}))
}

/// Copy a pack, what it requires and (the chosen part of) what it includes into the project. Conflicts are a 409 and
/// write nothing, unless `force=1`.
fn packs_add(
    app: &Arc<App>,
    req: Request,
    name: &str,
    only: Option<Vec<String>>,
    force: bool,
    clash: trumly_fs::pack::Clash,
) {
    let all = match trumly_fs::library::load(app.packs.as_deref()) {
        Ok(l) => l.packs(),
        Err(e) => return reply(req, 500, json!({"error": e})),
    };
    if !all.iter().any(|p| p.name == name) {
        return reply(req, 404, json!({"error": format!("no pack called {name}")}));
    }
    match trumly_fs::pack::add_tree_with(&all, name, only.as_deref(), &app.root(), force, clash) {
        Ok(done) => {
            app.refresh();
            let count = |f: fn(&trumly_fs::pack::Added) -> usize| done.iter().map(|(_, a)| f(a)).sum::<usize>();
            reply(
                req,
                200,
                json!({
                    "packs": done.iter().map(|(n, _)| n).collect::<Vec<_>>(),
                    "added": count(|a| a.added.len()), "updated": count(|a| a.updated.len()),
                    "no_longer_in_pack": done.iter().flat_map(|(_, a)| a.no_longer_in_pack.clone()).collect::<Vec<_>>(),
                    "kept": done.iter().flat_map(|(_, a)| a.kept.clone()).collect::<Vec<_>>(),
                    "skipped": done.iter().flat_map(|(_, a)| a.skipped.clone()).collect::<Vec<_>>(),
                    "replaced": done.iter().flat_map(|(_, a)| a.replaced.clone()).collect::<Vec<_>>(),
                    "removed": done.iter().flat_map(|(_, a)| a.removed.clone()).collect::<Vec<_>>(),
                }),
            )
        }
        Err(e) => reply(req, 409, json!({"error": e, "name_clash": e.contains("name clash")})),
    }
}

fn conflict(app: &App, req: Request, rel: Option<&str>) {
    let current = rel.and_then(|p| read_node(app, p).map(|(n, rev)| node_with_file(&n, p, &rev)));
    reply(req, 409, json!({"error": "the file changed on disk", "current": current}));
}

fn put_node(app: &Arc<App>, req: Request, id: &str, body: &str) {
    if !valid_id(id) {
        return reply(req, 400, json!({"error": "invalid id"}));
    }
    let Ok(Value::Object(payload)) = serde_json::from_str::<Value>(body) else {
        return reply(req, 400, json!({"error": "body must be a JSON object"}));
    };
    let node_json = payload.get("node").and_then(Value::as_object).cloned().unwrap_or_default();
    let body_text = node_json.get("body").and_then(Value::as_str).unwrap_or("").to_string();
    let node = match node_from_map(&node_json, body_text) {
        Ok(n) if n.id == id => n,
        Ok(_) => return reply(req, 400, json!({"error": "node id does not match the URL"})),
        Err(e) => return reply(req, 400, json!({"error": format!("invalid node: {}", e.as_str())})),
    };

    app.refresh();
    let existing = app.path_for_id(id);
    if existing.is_none() {
        let namespaced = node.kind.as_deref() == Some("pack") || node.fields.get("pack").is_some_and(|p| p.is_string());
        if !portable_id(id, namespaced) {
            return reply(
                req,
                400,
                json!({"error": "new ids must be lower-case letters, digits, '-' or '_' (max 100), so the file name works on every system"}),
            );
        }
        if app.state.lock().unwrap().files.keys().any(|p| stem(p).eq_ignore_ascii_case(id)) {
            return reply(
                req,
                400,
                json!({"error": "another file has this name apart from its capitals; they would collide on macOS and Windows"}),
            );
        }
    }
    let rel = existing.clone().unwrap_or_else(|| format!("{id}.md"));
    let base_rev = payload.get("base_rev").and_then(Value::as_str);
    let on_disk = app.rev_now(&rel);
    // An existing file may only be replaced by someone who has seen its current version.
    if on_disk.is_some() && on_disk.as_deref() != base_rev {
        return conflict(app, req, Some(&rel));
    }
    if on_disk.is_none() && base_rev.is_some() {
        return conflict(app, req, None); // it was deleted underneath the editor
    }

    let target = app.root().join(&rel);
    if let Err(e) = write_atomically(&target, &node_to_markdown(&node)) {
        return reply(req, 500, json!({"error": format!("write failed: {e}")}));
    }
    let rev = app.note_write(&rel);
    let version = app.state.lock().unwrap().version;
    reply(req, 200, json!({"path": rel, "rev": rev, "version": version}))
}

fn delete_node(app: &Arc<App>, req: Request, id: &str, base_rev: Option<String>) {
    if !valid_id(id) {
        return reply(req, 400, json!({"error": "invalid id"}));
    }
    app.refresh();
    let Some(rel) = app.path_for_id(id) else {
        return reply(req, 200, json!({"deleted": false})); // already gone: deleting is idempotent
    };
    if app.rev_now(&rel) != base_rev {
        return conflict(app, req, Some(&rel));
    }
    match fs::remove_file(app.root().join(&rel)) {
        Ok(()) => {
            app.note_delete(&rel);
            reply(req, 200, json!({"deleted": true}))
        }
        Err(e) => reply(req, 500, json!({"error": format!("delete failed: {e}")})),
    }
}

// ---------- startup ----------

/// Compares two tokens without stopping at the first difference, so the time a refusal takes says nothing about how much of a guess was right.
fn same_token(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        diff |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
    }
    diff == 0
}

fn random_token() -> String {
    let mut buf = [0u8; 16];
    let ok = fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut buf)).is_ok();
    if !ok {
        // No /dev/urandom (Windows): take the bytes from the standard library's hash seeds, which the operating system's
        // random number generator provides on every platform.
        use std::hash::{BuildHasher, Hasher};
        for chunk in buf.chunks_mut(8) {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u8(1);
            chunk.copy_from_slice(&h.finish().to_le_bytes());
        }
    }
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

/// Start the server on background threads and return where it is listening.
pub fn start(opts: Options) -> Result<Running, String> {
    let root = fs::canonicalize(&opts.root).map_err(|e| format!("cannot open {}: {e}", opts.root.display()))?;
    let server = Server::http(&opts.addr).map_err(|e| format!("cannot listen on {}: {e}", opts.addr))?;
    let addr = server.server_addr().to_ip().ok_or("server has no ip address")?;
    let token = opts.token.unwrap_or_else(random_token);
    let browse =
        opts.browse.as_deref().and_then(|b| fs::canonicalize(b).ok()).or_else(home_dir).unwrap_or_else(|| root.clone());
    let app = Arc::new(App {
        root: RwLock::new(root),
        browse,
        recent: opts.state.map(|d| d.join("recent.json")).or_else(recent_file),
        page: opts.page,
        packs: opts.packs,
        token: token.clone(),
        port: addr.port(),
        state: Mutex::new(Shared {
            files: BTreeMap::new(),
            gone: BTreeMap::new(),
            touched: BTreeMap::new(),
            version: 1,
        }),
        changed: Condvar::new(),
    });
    // The first scan is the baseline, not a change.
    let first = app.root();
    app.state.lock().unwrap().files = scan(&first);
    remember(app.recent.as_deref(), &first);

    let watcher = app.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(300));
            watcher.refresh();
        }
    });
    let acceptor = app.clone();
    thread::spawn(move || {
        for req in server.incoming_requests() {
            let app = acceptor.clone();
            thread::spawn(move || handle(&app, req)); // one thread per request, so a long poll blocks nobody
        }
    });
    Ok(Running { addr, token })
}

/// Run until the process is stopped.
pub fn serve(opts: Options) -> Result<(), String> {
    let running = start(opts)?;
    println!("trumly serve: http://{}/", running.addr);
    loop {
        thread::park();
    }
}

// ---------- opening another project folder ----------

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).and_then(|h| fs::canonicalize(h).ok())
}

fn recent_file() -> Option<PathBuf> {
    trumly_fs::telemetry::trumly_home().map(|h| h.join("recent.json"))
}

fn recent_read(file: Option<&Path>) -> Vec<String> {
    file.and_then(|f| fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok())
        .unwrap_or_default()
}

/// Put a folder at the front of the list of projects you opened lately (best effort: failing to save is fine).
fn remember(file: Option<&Path>, root: &Path) {
    let Some(f) = file else { return };
    let me = root.display().to_string();
    let mut list: Vec<String> = recent_read(Some(f)).into_iter().filter(|p| *p != me).collect();
    list.insert(0, me);
    list.truncate(8);
    if let Some(dir) = f.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(f, serde_json::to_string(&list).unwrap_or_default());
}

/// A folder we may look in: it exists and sits inside the browse root (so `..` and symlinks cannot leave it).
fn inside(app: &App, p: &Path) -> Option<PathBuf> {
    let c = fs::canonicalize(p).ok()?;
    (c.is_dir() && c.starts_with(&app.browse)).then_some(c)
}

fn md_count(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "md") && e.path().is_file())
                .count()
        })
        .unwrap_or(0)
}

fn dirs_list(app: &Arc<App>, req: Request, path: Option<String>) {
    let want = path.map(|p| percent_decode(&p)).filter(|p| !p.is_empty());
    let start = match want {
        Some(p) => PathBuf::from(p),
        None => app.root().parent().map(Path::to_path_buf).unwrap_or_else(|| app.browse.clone()),
    };
    let Some(dir) = inside(app, &start).or_else(|| Some(app.browse.clone())) else {
        return reply(req, 400, json!({"error": "cannot read that folder"}));
    };
    let mut dirs: Vec<Value> = fs::read_dir(&dir)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    (!name.starts_with('.') && name != "node_modules" && name != "target")
                        .then(|| json!({"name": name, "md": md_count(&e.path())}))
                })
                .collect()
        })
        .unwrap_or_default();
    dirs.sort_by(|a, b| {
        a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase())
    });
    let parent = dir.parent().filter(|p| p.starts_with(&app.browse)).map(|p| p.display().to_string());
    reply(
        req,
        200,
        json!({"path": dir.display().to_string(), "parent": parent, "home": app.browse.display().to_string(), "md": md_count(&dir), "dirs": dirs, "current": app.root().display().to_string()}),
    );
}

fn recent_list(app: &Arc<App>, req: Request) {
    let list: Vec<Value> = recent_read(app.recent.as_deref())
        .into_iter()
        .filter(|p| inside(app, Path::new(p)).is_some())
        .map(|p| {
            let name = Path::new(&p).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.clone());
            json!({"path": p, "name": name})
        })
        .collect();
    reply(req, 200, json!({"recent": list, "current": app.root().display().to_string()}));
}

/// Switch the project the server works on. `{"path": "...", "create": true}` makes the folder first if it is not there.
fn open_folder(app: &Arc<App>, req: Request, body: &str) {
    let Ok(v) = serde_json::from_str::<Value>(body) else { return reply(req, 400, json!({"error": "bad json"})) };
    let Some(path) = v["path"].as_str().filter(|p| !p.is_empty()) else {
        return reply(req, 400, json!({"error": "path is required"}));
    };
    let want = PathBuf::from(path);
    if v["create"].as_bool() == Some(true) && !want.exists() {
        // only inside the browse root, and one plain folder name at a time
        let parent_ok = want.parent().and_then(|p| inside(app, p)).is_some();
        let name_ok = want
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| !n.is_empty() && !n.starts_with('.') && !n.contains(['/', '\\']));
        if !parent_ok || !name_ok {
            return reply(req, 400, json!({"error": "cannot make a folder there"}));
        }
        if let Err(e) = fs::create_dir(&want) {
            return reply(req, 500, json!({"error": format!("cannot make the folder: {e}")}));
        }
    }
    let Some(dir) = inside(app, &want) else {
        return reply(req, 400, json!({"error": "that is not a folder you can open"}));
    };
    {
        let mut root = app.root.write().unwrap();
        *root = dir.clone();
        let mut s = app.state.lock().unwrap();
        // a new baseline; the version only ever goes up, so editors polling for changes notice and reload
        s.files = scan(&dir);
        s.gone.clear();
        s.touched.clear();
        s.version += 1;
    }
    app.changed.notify_all();
    remember(app.recent.as_deref(), &dir);
    let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    reply(req, 200, json!({"name": name, "root": dir.display().to_string()}));
}
