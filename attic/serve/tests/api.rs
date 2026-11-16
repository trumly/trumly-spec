//! Drives the real server over HTTP against a scratch copy of the portal project.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use serde_json::{Value, json};
use trumly_serve::{Options, start};

const TOKEN: &str = "test-token";

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trumly-serve-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/portal");
    fs::create_dir_all(&dir).unwrap();
    for e in fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
    }
    dir
}

struct Api {
    base: String,
    port: u16,
}

impl Api {
    fn new(dir: &Path) -> Api {
        let run = start(Options {
            root: dir.into(),
            addr: "127.0.0.1:0".into(),
            page: None,
            token: Some(TOKEN.into()),
            packs: None,
            state: None,
            browse: None,
        })
        .unwrap();
        Api { base: format!("http://127.0.0.1:{}", run.addr.port()), port: run.addr.port() }
    }
    fn call(&self, method: &str, path: &str, body: Option<Value>, token: Option<&str>) -> (u16, Value) {
        let mut req = ureq::request(method, &format!("{}{path}", self.base));
        if let Some(t) = token {
            req = req.set("X-Trumly-Token", t);
        }
        let res = match body {
            Some(b) => req.send_string(&b.to_string()),
            None => req.call(),
        };
        let res = match res {
            Ok(r) => r,
            Err(ureq::Error::Status(_, r)) => r,
            Err(e) => panic!("{e}"),
        };
        let status = res.status();
        (status, serde_json::from_str(&res.into_string().unwrap()).unwrap_or(Value::Null))
    }
    fn get(&self, path: &str) -> (u16, Value) {
        self.call("GET", path, None, Some(TOKEN))
    }
}

fn node<'a>(model: &'a Value, id: &str) -> &'a Value {
    model["nodes"].as_array().unwrap().iter().find(|n| n["id"] == id).unwrap()
}

#[test]
fn model_lists_every_node_with_path_and_rev() {
    let dir = scratch("model");
    let api = Api::new(&dir);
    let (status, model) = api.get("/api/model");
    assert_eq!(status, 200);
    assert_eq!(model["nodes"].as_array().unwrap().len(), 21);
    let n = node(&model, "credit-ledger");
    assert_eq!(n["path"], "credit-ledger.md");
    assert!(n["rev"].as_str().unwrap().contains('-'));
}

#[test]
fn saving_a_node_rewrites_its_file_and_keeps_the_body() {
    let dir = scratch("save");
    let api = Api::new(&dir);
    // give the file a prose body first, as a human or agent would
    let path = dir.join("credit-ledger.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{}\n# Notes\n\nAppend only.\n", text.trim_end())).unwrap();
    sleep(Duration::from_millis(50));

    let (_, model) = api.get("/api/model");
    let mut n = node(&model, "credit-ledger").clone();
    assert_eq!(n["body"], "# Notes\n\nAppend only.");
    let rev = n["rev"].clone();
    n["title"] = json!("Credit ledger (renamed)");
    let (status, saved) =
        api.call("PUT", "/api/nodes/credit-ledger", Some(json!({"node": n, "base_rev": rev})), Some(TOKEN));
    assert_eq!(status, 200, "{saved}");

    let on_disk = fs::read_to_string(&path).unwrap();
    assert!(on_disk.contains("title: Credit ledger (renamed)"), "{on_disk}");
    assert!(on_disk.ends_with("# Notes\n\nAppend only.\n"), "{on_disk}");
    assert!(on_disk.starts_with("---\nid: credit-ledger\n"));
}

#[test]
fn a_stale_save_is_refused_and_nothing_is_overwritten() {
    let dir = scratch("conflict");
    let api = Api::new(&dir);
    let (_, model) = api.get("/api/model");
    let mut n = node(&model, "credit-ledger").clone();
    let stale = n["rev"].clone();

    sleep(Duration::from_millis(20));
    let path = dir.join("credit-ledger.md");
    fs::write(&path, fs::read_to_string(&path).unwrap().replace("Credit ledger", "Edited elsewhere")).unwrap();

    n["title"] = json!("Mine");
    let (status, body) =
        api.call("PUT", "/api/nodes/credit-ledger", Some(json!({"node": n, "base_rev": stale})), Some(TOKEN));
    assert_eq!(status, 409);
    assert_eq!(body["current"]["title"], "Edited elsewhere");
    assert!(fs::read_to_string(&path).unwrap().contains("Edited elsewhere"));
}

#[test]
fn creating_and_deleting_nodes() {
    let dir = scratch("create");
    let api = Api::new(&dir);
    let new = json!({"id": "fresh", "type": "entity", "title": "Fresh", "fields": {"n": 1}, "links": [], "body": ""});
    let (status, saved) = api.call("PUT", "/api/nodes/fresh", Some(json!({"node": new})), Some(TOKEN));
    assert_eq!(status, 200, "{saved}");
    assert!(dir.join("fresh.md").exists());

    // creating over an existing file without a rev must not clobber it
    let (status, _) = api.call("PUT", "/api/nodes/fresh", Some(json!({"node": new})), Some(TOKEN));
    assert_eq!(status, 409);

    let rev = saved["rev"].as_str().unwrap();
    let (status, _) = api.call("DELETE", &format!("/api/nodes/fresh?base_rev={rev}"), None, Some(TOKEN));
    assert_eq!(status, 200);
    assert!(!dir.join("fresh.md").exists());
}

#[test]
fn outside_edits_arrive_through_changes() {
    let dir = scratch("changes");
    let api = Api::new(&dir);
    let (_, info) = api.get("/api/info");
    let since = info["version"].as_u64().unwrap();

    fs::write(dir.join("extra.md"), "---\nid: extra\ntype: note\n---\n\nHello\n").unwrap();
    fs::remove_file(dir.join("vercel.md")).unwrap();

    let (status, changes) = api.get(&format!("/api/changes?since={since}")); // long-polls until the watcher notices
    assert_eq!(status, 200);
    assert!(changes["version"].as_u64().unwrap() > since);
    let mut seen_new = changes["nodes"].as_array().unwrap().iter().any(|n| n["id"] == "extra");
    let mut seen_gone = changes["removed"].as_array().unwrap().iter().any(|n| n == "vercel");
    if !(seen_new && seen_gone) {
        // the watcher may have noticed the two files in separate passes
        let (_, more) = api.get(&format!("/api/changes?since={since}"));
        seen_new |= more["nodes"].as_array().unwrap().iter().any(|n| n["id"] == "extra");
        seen_gone |= more["removed"].as_array().unwrap().iter().any(|n| n == "vercel");
    }
    assert!(seen_new && seen_gone);
}

#[test]
fn our_own_writes_do_not_echo_back_as_outside_changes() {
    let dir = scratch("echo");
    let api = Api::new(&dir);
    let (_, model) = api.get("/api/model");
    let mut n = node(&model, "portal").clone();
    n["title"] = json!("Portal 2");
    let (_, saved) =
        api.call("PUT", "/api/nodes/portal", Some(json!({"node": n.clone(), "base_rev": n["rev"]})), Some(TOKEN));
    let after_save = saved["version"].as_u64().unwrap();
    sleep(Duration::from_millis(800)); // several watcher passes
    let (_, info) = api.get("/api/info");
    assert_eq!(info["version"].as_u64().unwrap(), after_save, "the watcher re-announced our own write");
}

#[test]
fn security() {
    let dir = scratch("security");
    let api = Api::new(&dir);
    assert_eq!(api.call("GET", "/api/model", None, None).0, 403, "no token");
    assert_eq!(api.call("GET", "/api/model", None, Some("wrong")).0, 403, "wrong token");
    assert_eq!(api.call("PUT", "/api/nodes/x", Some(json!({})), None).0, 403, "write without token");

    for bad in ["..%2Fescape", "%2Fetc%2Fpasswd", ".hidden", "a%5Cb"] {
        let node = json!({"id": "x", "type": "t"});
        let (status, _) = api.call("PUT", &format!("/api/nodes/{bad}"), Some(json!({"node": node})), Some(TOKEN));
        assert_eq!(status, 400, "id {bad} must be refused");
    }
    assert!(!dir.parent().unwrap().join("escape.md").exists());

    // a wrong Host header (DNS rebinding) is refused even with the token
    let res =
        ureq::get(&format!("{}/api/model", api.base)).set("X-Trumly-Token", TOKEN).set("Host", "evil.example").call();
    assert!(matches!(res, Err(ureq::Error::Status(403, _))), "{res:?}");
    let _ = api.port;
}

#[test]
fn the_page_gets_the_token_injected() {
    let dir = scratch("page");
    let page = dir.join("page.html");
    fs::write(&page, "<title>t</title>hello").unwrap();
    let run = start(Options {
        root: dir.clone(),
        addr: "127.0.0.1:0".into(),
        page: Some(page),
        token: Some(TOKEN.into()),
        packs: None,
        state: None,
        browse: None,
    })
    .unwrap();
    let html = ureq::get(&format!("http://127.0.0.1:{}/", run.addr.port())).call().unwrap().into_string().unwrap();
    assert!(html.starts_with(&format!("<meta name=\"trumly-token\" content=\"{TOKEN}\">")) && html.contains("hello"));
}

#[test]
fn view_state_is_stored_in_the_folder_and_does_not_touch_the_model() {
    let dir = scratch("view");
    let api = Api::new(&dir);
    let (status, v) = api.get("/api/view");
    assert_eq!((status, v["exists"].clone()), (200, json!(false)));
    assert_eq!(v["view"], json!({}));

    let view = json!({"pins": {"web-app": {"c": 1, "r": 2}}, "views": {"wide": {"name": "Wide", "angle": 10, "pitch": 0.5, "gap": 60}}});
    let (status, _) = api.call("PUT", "/api/view", Some(view.clone()), Some(TOKEN));
    assert_eq!(status, 200);
    assert!(dir.join(".trumly/view.json").exists());
    let (_, back) = api.get("/api/view");
    assert_eq!(back["view"], view);
    assert_eq!(back["exists"], json!(true));

    // the project is unchanged: still 21 nodes, no stray files treated as nodes, nothing left half-written
    let (_, model) = api.get("/api/model");
    assert_eq!(model["nodes"].as_array().unwrap().len(), 21);
    let leftovers: Vec<_> = fs::read_dir(dir.join(".trumly"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers, vec!["view.json".to_string()]);
}

#[test]
fn view_state_needs_the_token_and_a_json_object() {
    let dir = scratch("view-bad");
    let api = Api::new(&dir);
    assert_eq!(api.call("GET", "/api/view", None, None).0, 403);
    assert_eq!(api.call("PUT", "/api/view", Some(json!({"a": 1})), Some("wrong")).0, 403);
    assert_eq!(api.call("PUT", "/api/view", Some(json!([1, 2])), Some(TOKEN)).0, 400);
    assert!(!dir.join(".trumly").exists());
}

#[test]
fn new_ids_must_be_portable_file_names() {
    let dir = scratch("portable");
    let api = Api::new(&dir);
    let n = |id: &str| json!({"id": id, "type": "component", "title": "x", "fields": {}, "links": [], "body": ""});
    for bad in ["Has-Caps", "con", "nul", "com1", "-lead", "trail-", "a.b", "a b", "ünï", "a--b", &"a".repeat(101)] {
        let (status, _) = api.call(
            "PUT",
            &format!("/api/nodes/{}", bad.replace(' ', "%20")),
            Some(json!({"node": n(bad)})),
            Some(TOKEN),
        );
        assert_eq!(status, 400, "{bad} should be refused");
    }
    assert_eq!(
        api.call("PUT", "/api/nodes/component-api_v2", Some(json!({"node": n("component-api_v2")})), Some(TOKEN)).0,
        200
    );
    // `--` is the pack namespace separator: only a node that says it comes from a pack may use it
    let packed = json!({"id": "sdl--type-x", "type": "type", "title": "x", "fields": {"pack": "system-design-lite@0.2.1"}, "links": [], "body": ""});
    assert_eq!(api.call("PUT", "/api/nodes/sdl--type-x", Some(json!({"node": packed})), Some(TOKEN)).0, 200);
    let manifest = json!({"id": "mine--pack", "type": "pack", "title": "mine", "fields": {"name": "mine", "namespace": "mine", "version": "0.1.0"}, "links": [], "body": ""});
    assert_eq!(api.call("PUT", "/api/nodes/mine--pack", Some(json!({"node": manifest})), Some(TOKEN)).0, 200);
}

#[test]
fn a_name_that_differs_only_by_case_is_refused() {
    let dir = scratch("casefold");
    std::fs::write(dir.join("Legacy-Node.md"), "---\nid: Legacy-Node\ntype: component\ntitle: Old\n---\n").unwrap();
    let api = Api::new(&dir);
    let n = json!({"id": "legacy-node", "type": "component", "title": "x", "fields": {}, "links": [], "body": ""});
    assert_eq!(api.call("PUT", "/api/nodes/legacy-node", Some(json!({"node": n})), Some(TOKEN)).0, 400);
}

#[test]
fn packs_can_be_listed_and_added_through_the_api() {
    let dir = scratch("packs");
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fs/tests/fixtures/packs");
    let run = start(Options {
        root: dir.clone(),
        addr: "127.0.0.1:0".into(),
        page: None,
        token: Some(TOKEN.into()),
        packs: Some(packs),
        state: None,
        browse: None,
    })
    .unwrap();
    let api = Api { base: format!("http://127.0.0.1:{}", run.addr.port()), port: run.addr.port() };
    let (status, list) = api.get("/api/packs");
    assert_eq!(status, 200);
    let names: Vec<&str> = list["packs"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"system-design") && names.contains(&"adr"));
    assert!(list["packs"][0]["installed"].is_null());

    // adding a pack brings what it requires: ux needs shared, and the ids cannot clash with the portal's
    let (status, body) = api.call("POST", "/api/packs/ux", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["packs"], json!(["shared", "ux"]));
    assert!(dir.join("shr--pack.md").exists() && dir.join("ux--pack.md").exists());
    let (_, list) = api.get("/api/packs");
    let ux = list["packs"].as_array().unwrap().iter().find(|p| p["name"] == "ux").unwrap();
    assert!(ux["installed"].is_string() && ux["namespace"] == "ux");
    let coll = list["packs"].as_array().unwrap().iter().find(|p| p["name"] == "design-suite").unwrap();
    assert!(coll["includes"].as_array().unwrap().len() == 12);

    // choosing parts of a collection; a type of the same name already in the project is refused, then forced
    fs::write(dir.join("type-decision.md"), "---\nid: type-decision\ntype: type\nfields: { name: decision }\n---\n")
        .unwrap();
    let (status, body) = api.call("POST", "/api/packs/design-suite?only=adr", None, Some(TOKEN));
    assert_eq!(status, 409, "{body}");
    assert!(body["error"].as_str().unwrap().contains("the type 'decision'"));
    assert!(!dir.join("adr--pack.md").exists());
    let (status, body) = api.call("POST", "/api/packs/design-suite?only=adr&force=1", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert!(dir.join("adr--pack.md").exists() && !dir.join("dm--pack.md").exists());

    // detaching lets go of a pack and keeps its definitions as ordinary nodes
    let (status, body) = api.call("POST", "/api/packs/shared/detach", None, Some(TOKEN));
    assert_eq!(status, 409, "{body}"); // ux still needs it
    let (status, body) = api.call("POST", "/api/packs/ux/detach", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert!(body["kept"].as_array().unwrap().len() > 2);
    assert!(!dir.join("ux--pack.md").exists());
    let (status, body) = api.call("POST", "/api/packs/adr/detach", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    let (status, body) = api.call("POST", "/api/packs/shared/detach", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    let (_, status_list) = api.get("/api/packs/status");
    assert!(!status_list["packs"].as_array().unwrap().iter().any(|p| p["name"] == "ux"));

    let (status, _) = api.call("POST", "/api/packs/nope", None, Some(TOKEN));
    assert_eq!(status, 404);
    let (status, _) = api.call("POST", "/api/packs/adr", None, None);
    assert_eq!(status, 403);
    let _ = api.port;
}

#[test]
fn every_pack_lists_its_examples_and_one_opens_in_full() {
    let dir = scratch("samples");
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fs/tests/fixtures/packs");
    let run = start(Options {
        root: dir.clone(),
        addr: "127.0.0.1:0".into(),
        page: None,
        token: Some(TOKEN.into()),
        packs: Some(packs),
        state: None,
        browse: None,
    })
    .unwrap();
    let api = Api { base: format!("http://127.0.0.1:{}", run.addr.port()), port: run.addr.port() };
    let (_, list) = api.get("/api/packs");
    for p in list["packs"].as_array().unwrap() {
        assert!(
            p["samples"].as_array().unwrap().iter().any(|s| s["kind"] == "example"),
            "{} has no example",
            p["name"]
        );
    }
    let (status, one) = api.get("/api/packs/system-design/patterns/publish-subscribe");
    assert_eq!(status, 200, "{one}");
    assert_eq!(one["root"], "topic");
    assert!(one["nodes"].as_array().unwrap().len() >= 4);
    assert_eq!(api.get("/api/packs/system-design/examples/nope").0, 404);
}

#[test]
fn pack_status_and_removal_through_the_api() {
    let dir = scratch("packrm");
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fs/tests/fixtures/packs");
    let run = start(Options {
        root: dir.clone(),
        addr: "127.0.0.1:0".into(),
        page: None,
        token: Some(TOKEN.into()),
        packs: Some(packs),
        state: None,
        browse: None,
    })
    .unwrap();
    let api = Api { base: format!("http://127.0.0.1:{}", run.addr.port()), port: run.addr.port() };
    assert_eq!(api.call("POST", "/api/packs/adr", None, Some(TOKEN)).0, 200);
    let (_, st) = api.get("/api/packs/status");
    let adr =
        |st: &serde_json::Value| st["packs"].as_array().unwrap().iter().find(|p| p["name"] == "adr").unwrap().clone();
    assert_eq!(adr(&st)["changed"], json!([]));
    // you edit one of its nodes: status says so, and a plain removal is refused
    let f = dir.join("adr--rule-governs.md");
    fs::write(
        &f,
        fs::read_to_string(&f)
            .unwrap()
            .replace("An accepted decision governs something", "An accepted decision governs two things"),
    )
    .unwrap();
    sleep(Duration::from_millis(50));
    let (_, st) = api.get("/api/packs/status");
    assert_eq!(adr(&st)["changed"], json!(["adr--rule-governs"]));
    let (status, body) = api.call("DELETE", "/api/packs/adr", None, Some(TOKEN));
    assert_eq!(status, 409, "{body}");
    assert!(dir.join("adr--pack.md").exists());
    let (status, body) = api.call("DELETE", "/api/packs/adr?force=1", None, Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert!(!dir.join("adr--pack.md").exists());
    assert_eq!(api.call("DELETE", "/api/packs/adr", None, None).0, 403);
    let _ = api.port;
}

#[test]
fn the_collector_keeps_only_well_formed_events() {
    use std::io::Read;
    let file = std::env::temp_dir().join(format!("trumly-collector-test-{}.jsonl", std::process::id()));
    let _ = std::fs::remove_file(&file);
    let f = file.clone();
    std::thread::spawn(move || {
        let _ = trumly_serve::collector::run("127.0.0.1:47931", f);
    });
    std::thread::sleep(std::time::Duration::from_millis(400));
    let good = json!({"event": "pack_added", "ts": "2026-10-21T10:00:00Z", "install": "abc", "version": "0.0.1", "os": "linux", "src": "editor", "props": {"pack": "decisions"}});
    let bad = json!({"event": "Has Spaces", "ts": "x", "install": "abc", "version": "1", "os": "o", "src": "s"});
    let nested = json!({"event": "x", "ts": "x", "install": "abc", "version": "1", "os": "o", "src": "s", "props": {"a": {"b": 1}}});
    let r: serde_json::Value = ureq::post("http://127.0.0.1:47931/v1/events")
        .send_json(json!({"events": [good, bad, nested]}))
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(r["accepted"], 1);
    assert_eq!(r["dropped"], 2);
    let s: serde_json::Value = ureq::get("http://127.0.0.1:47931/v1/summary").call().unwrap().into_json().unwrap();
    assert_eq!(s["events"], 1);
    assert_eq!(s["by_event"]["pack_added"], 1);
    let mut text = String::new();
    std::fs::File::open(&file).unwrap().read_to_string(&mut text).unwrap();
    assert_eq!(text.lines().count(), 1);
    let _ = std::fs::remove_file(&file);
}

#[test]
fn another_model_folder_can_be_browsed_opened_and_made_from_the_api() {
    let base = std::env::temp_dir().join(format!("trumly-serve-test-open-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let (a, b, empty) = (base.join("alpha"), base.join("beta"), base.join("plain"));
    for d in [&a, &b, &empty] {
        fs::create_dir_all(d).unwrap();
    }
    fs::write(a.join("one.md"), "---\nid: one\ntype: thing\n---\n").unwrap();
    fs::write(b.join("two.md"), "---\nid: two\ntype: thing\n---\n").unwrap();
    fs::write(b.join("three.md"), "---\nid: three\ntype: thing\n---\n").unwrap();
    fs::create_dir_all(base.join(".hidden")).unwrap();
    let home = std::env::temp_dir().join(format!("trumly-serve-test-open-home-{}", std::process::id()));
    let run = start(Options {
        root: a.clone(),
        addr: "127.0.0.1:0".into(),
        page: None,
        token: Some(TOKEN.into()),
        packs: None,
        state: Some(home.clone()),
        browse: Some(base.clone()),
    })
    .unwrap();
    let api = Api { base: format!("http://127.0.0.1:{}", run.addr.port()), port: run.addr.port() };

    // it lists the folder next to the current project, with how many notes each holds, and hides dotfolders
    let (status, list) = api.get("/api/dirs");
    assert_eq!(status, 200, "{list}");
    let names: Vec<&str> = list["dirs"].as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["alpha", "beta", "plain"]);
    assert_eq!(list["dirs"][1]["md"], 2);
    assert!(list["parent"].is_null(), "the browse root is the top");

    // switching changes what the API serves
    let (status, body) = api.call("POST", "/api/open", Some(json!({"path": b.display().to_string()})), Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert_eq!(api.get("/api/info").1["name"], "beta");
    let ids: Vec<String> = api.get("/api/model").1["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"two".to_string()) && !ids.contains(&"one".to_string()));
    // writes now land in the new folder
    let (status, _) = api.call("PUT", "/api/nodes/four", Some(json!({"node": {"id": "four", "type": "thing", "title": "Four", "fields": {"n": 1}, "links": [], "body": ""}})), Some(TOKEN));
    assert_eq!(status, 200);
    assert!(b.join("four.md").exists() && !a.join("four.md").exists());

    // it remembers recent projects, newest first
    let (_, recent) = api.get("/api/recent");
    assert_eq!(recent["recent"][0]["name"], "beta");

    // a new folder can be made inside the browse root, and opened
    let fresh = base.join("fresh");
    let (status, body) =
        api.call("POST", "/api/open", Some(json!({"path": fresh.display().to_string(), "create": true})), Some(TOKEN));
    assert_eq!(status, 200, "{body}");
    assert!(fresh.is_dir() && api.get("/api/info").1["name"] == "fresh");

    // and nothing outside it: not by path, not by `..`, not by making folders there, not without the token
    let outside = std::env::temp_dir().join(format!("trumly-serve-test-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).unwrap();
    for p in [outside.display().to_string(), format!("{}/../", base.display())] {
        let (status, _) = api.call("POST", "/api/open", Some(json!({"path": p})), Some(TOKEN));
        assert_eq!(status, 400, "opened outside the browse root");
    }
    let (status, _) = api.call(
        "POST",
        "/api/open",
        Some(json!({"path": outside.join("new").display().to_string(), "create": true})),
        Some(TOKEN),
    );
    assert_eq!(status, 400);
    assert!(!outside.join("new").exists());
    assert_eq!(api.call("POST", "/api/open", Some(json!({"path": a.display().to_string()})), None).0, 403);
    assert_eq!(api.get("/api/info").1["name"], "fresh", "refused requests change nothing");
}
