//! Anonymous usage data, off until the person says yes.
//!
//! What is kept: a random install id, the version, the OS, the name of what was done (`pack_added`, `check_run`) and
//! rounded counts. Never ids, titles, field values, file paths or project content. Everything is written to a local queue
//! first so `trumly telemetry show` can display exactly what would be sent; sending is done elsewhere (the editor, or
//! nothing at all) and a down endpoint costs nothing but a retry later.
//!
//! State lives in `~/.trumly/telemetry.json` (or `$TRUMLY_HOME`). `DO_NOT_TRACK=1` or `TRUMLY_TELEMETRY=off` always wins.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::{Value, json};

/// Where trumly keeps per-user files: `$TRUMLY_HOME`, else `~/.trumly`.
pub fn trumly_home() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("TRUMLY_HOME") {
        return Some(PathBuf::from(h));
    }
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(|h| PathBuf::from(h).join(".trumly"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// never asked: nothing is recorded, and the editor should ask
    Unset,
    On,
    Off,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Unset => "unset",
            State::On => "on",
            State::Off => "off",
        }
    }
    pub fn parse(s: &str) -> Option<State> {
        match s {
            "unset" => Some(State::Unset),
            "on" => Some(State::On),
            "off" => Some(State::Off),
            _ => None,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    trumly_home().map(|h| h.join("telemetry.json"))
}

fn queue_path() -> Option<PathBuf> {
    trumly_home().map(|h| h.join("telemetry-queue.jsonl"))
}

fn read_config() -> Value {
    config_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}))
}

/// Switched off from outside, whatever the file says.
pub fn forced_off() -> bool {
    std::env::var("DO_NOT_TRACK").is_ok_and(|v| !v.is_empty() && v != "0")
        || std::env::var("TRUMLY_TELEMETRY").is_ok_and(|v| v.eq_ignore_ascii_case("off"))
}

/// What the person chose (ignoring the environment overrides).
pub fn chosen() -> State {
    read_config().get("state").and_then(Value::as_str).and_then(State::parse).unwrap_or(State::Unset)
}

/// What is in force: `Off` when forced off.
pub fn state() -> State {
    if forced_off() { State::Off } else { chosen() }
}

/// A random id for this install. Made only when someone turns telemetry on.
fn new_install_id() -> String {
    let mut bytes = [0u8; 16];
    let ok = fs::File::open("/dev/urandom").and_then(|mut f| std::io::Read::read_exact(&mut f, &mut bytes)).is_ok();
    if !ok {
        use std::hash::{BuildHasher, Hasher};
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
        h.write_u32(std::process::id());
        let a = h.finish();
        h.write_u64(a);
        bytes[..8].copy_from_slice(&a.to_le_bytes());
        bytes[8..].copy_from_slice(&h.finish().to_le_bytes());
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The install id, if telemetry has been turned on at some point.
pub fn install_id() -> Option<String> {
    read_config().get("install").and_then(Value::as_str).map(str::to_string)
}

/// Where events go when nothing says otherwise. Events are only ever sent once the person has said yes (see `State`), and `trumly telemetry show` prints exactly what would go.
pub const DEFAULT_ENDPOINT: &str = "https://telemetry.trumly.co/v1/events";

/// Where events go: `$TRUMLY_TELEMETRY_URL`, else `endpoint` in the config file, else trumly's own collector. `TRUMLY_TELEMETRY_URL=off` (or an empty `endpoint`) keeps them on this computer.
pub fn endpoint() -> Option<String> {
    let pick = std::env::var("TRUMLY_TELEMETRY_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| read_config().get("endpoint").and_then(Value::as_str).map(str::to_string));
    match pick.as_deref() {
        Some("off" | "none") => None,
        Some(u) => Some(u.to_string()),
        None => Some(DEFAULT_ENDPOINT.to_string()),
    }
}

pub fn set_state(s: State) -> Result<(), String> {
    let path = config_path().ok_or("no home folder")?;
    let mut cfg = read_config();
    if !cfg.is_object() {
        cfg = json!({});
    }
    cfg["state"] = json!(s.as_str());
    if s == State::On && cfg.get("install").and_then(Value::as_str).is_none() {
        cfg["install"] = json!(new_install_id());
    }
    if s == State::Off {
        // turning it off also forgets the id and anything not yet sent
        if let Some(o) = cfg.as_object_mut() {
            o.remove("install");
        }
        if let Some(q) = queue_path() {
            let _ = fs::remove_file(q);
        }
    }
    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("cannot create {}: {e}", path.parent().unwrap().display()))?;
    fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// `2026-10-21T17:40:00Z`, rounded down to the minute.
pub fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rem) = ((secs / 86400) as i64, secs % 86400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:00Z", rem / 3600, rem % 3600 / 60)
}

/// A count rounded into a band, so the number itself says little.
pub fn bucket(n: usize) -> &'static str {
    match n {
        0 => "0",
        1..=9 => "1-9",
        10..=49 => "10-49",
        50..=199 => "50-199",
        _ => "200+",
    }
}

/// The event as it would be sent.
pub fn event(name: &str, src: &str, props: Value) -> Option<Value> {
    Some(
        json!({"event": name, "ts": now_iso(), "install": install_id()?, "version": env!("CARGO_PKG_VERSION"), "os": std::env::consts::OS, "src": src, "props": props}),
    )
}

const QUEUE_LIMIT: usize = 1000;

/// Record an event if telemetry is on. Never fails and never blocks on the network.
pub fn track(name: &str, src: &str, props: Value) {
    if state() != State::On {
        return;
    }
    let (Some(ev), Some(path)) = (event(name, src, props), queue_path()) else { return };
    push(&path, &ev);
}

fn push(path: &PathBuf, ev: &Value) {
    let _ = fs::create_dir_all(path.parent().unwrap());
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{ev}");
    }
    // keep the queue bounded: when nobody is sending, the oldest events go
    if let Ok(text) = fs::read_to_string(path) {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() > QUEUE_LIMIT {
            let _ = fs::write(path, lines[lines.len() - QUEUE_LIMIT..].join("\n") + "\n");
        }
    }
}

/// Add an event someone else made (the editor's) to the queue, if telemetry is on. Only the fields we define are kept.
pub fn record(name: &str, props: Value) -> bool {
    if state() != State::On
        || name.is_empty()
        || name.len() > 48
        || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return false;
    }
    let props = match props {
        Value::Object(m)
            if m.len() <= 12
                && m.iter().all(|(k, v)| {
                    k.len() <= 32 && (v.is_boolean() || v.is_number() || v.as_str().is_some_and(|s| s.len() <= 48))
                }) =>
        {
            Value::Object(m)
        }
        _ => json!({}),
    };
    track(name, "editor", props);
    true
}

/// The first `max` queued events, oldest first.
pub fn queued(max: usize) -> Vec<Value> {
    queue_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|t| t.lines().filter_map(|l| serde_json::from_str(l).ok()).take(max).collect())
        .unwrap_or_default()
}

pub fn queued_count() -> usize {
    queue_path().and_then(|p| fs::read_to_string(p).ok()).map_or(0, |t| t.lines().count())
}

/// Drop the first `n` events once they have been sent.
pub fn ack(n: usize) {
    let Some(path) = queue_path() else { return };
    let Ok(text) = fs::read_to_string(&path) else { return };
    let rest: Vec<&str> = text.lines().skip(n).collect();
    if rest.is_empty() {
        let _ = fs::remove_file(&path);
    } else {
        let _ = fs::write(&path, rest.join("\n") + "\n");
    }
}
