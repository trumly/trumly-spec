//! Packs: folders of ordinary node files (planes, types, link types, rules) that can be added to a project.
//!
//! A pack folder holds one `pack` node (the manifest) and the nodes it provides. Every provided node carries
//! `fields.pack: <name>@<version>`, so once copied into a project it is still known to come from the pack.
//! Adding is a copy with checks: required packs must be there, and a node that already exists is only
//! replaced if it came from the same pack (an upgrade). Anything else is a conflict, and nothing is written.
//!
//! The manifest kept in a project also records `installed`: a hash of every node as the pack provided it. That is
//! what lets us tell a node you changed from one you did not, so an upgrade replaces what is untouched, keeps what
//! you edited, and a removal does not throw your work away.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use trumly_core::format::node_to_markdown;
use trumly_core::model::Node;
use trumly_core::parse::parse_node;
use trumly_core::report::node_to_json;

#[derive(Debug, Clone)]
pub struct Pack {
    pub name: String,
    /// the name to show people ("Data model"): the pack node's title. `name` stays the identifier
    pub title: String,
    pub version: String,
    pub description: String,
    /// the short prefix of every node id this pack provides: `<namespace>--<local id>`
    pub namespace: String,
    /// who may use the definitions and how: an SPDX identifier (`Apache-2.0`, `CC-BY-4.0`) or expression, or `LicenseRef-...` for your own terms
    pub license: String,
    /// who made and answers for the pack
    pub publisher: String,
    /// `core` (the bundle every project starts from, the only packs built into the binary) or `enrichment` (everything else; the default)
    pub role: String,
    /// the core packs an enrichment pack adds detail to (`design`, `plan`...); it helps people and agents choose, it is not a dependency
    pub enriches: Vec<String>,
    /// one sentence on when to reach for this pack, shown wherever packs are listed
    pub use_when: String,
    /// the pattern (a folder under `patterns/`) to lift as "another one" when the pack is used many times in a project: a flow, a lifecycle
    pub default_pattern: Option<String>,
    /// the patterns offered with "Include starter patterns" when the pack is added; the default pattern alone when empty
    pub starter_patterns: Vec<String>,
    /// packs that must already be in the project (or come along in the same add)
    pub requires: Vec<String>,
    /// packs this one works with when the project has them: never installed by adding this pack, never needed for it to load
    pub optional: Vec<String>,
    /// child packs this one bundles: adding it brings them in too (a collection has only a manifest)
    pub includes: Vec<String>,
    /// (node id, file text), the manifest included
    pub files: Vec<(String, String)>,
    /// worked examples and reusable patterns that show the pack in use (folders `examples/` and `patterns/`)
    pub samples: Vec<Sample>,
}

/// A small project that shows a pack in use. An `example` is meant to be read or used as a starting point; a `pattern`
/// is a fragment you lift into a project of your own, and says which node it hangs from.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// `example` or `pattern`
    pub kind: String,
    /// the folder name
    pub name: String,
    pub title: String,
    pub description: String,
    /// `starter` (copy it and carry on) or `inspiration` (flick through it)
    pub usage: String,
    /// packs it needs to check clean, the owning pack included
    pub requires: Vec<String>,
    /// packs it also uses when the project has them: the nodes that need one are left out of a project without it
    pub optional: Vec<String>,
    /// patterns only: the node the rest hangs from
    pub root: Option<String>,
    /// patterns only: the id prefix used when it is inserted
    pub prefix: Option<String>,
    /// (node id, file text) of the nodes; the manifest is not among them
    pub files: Vec<(String, String)>,
}

impl Pack {
    pub fn tag(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// A short line per node the pack provides (the manifest left out), for looking inside a pack before adding it.
    pub fn contents(&self) -> Vec<serde_json::Value> {
        let text = |n: &Node, k: &str| n.fields.get(k).and_then(|v| v.as_str()).map(str::to_string);
        let mut out: Vec<serde_json::Value> = self
            .files
            .iter()
            .filter_map(|(_, t)| parse_node(t).ok())
            .filter(|n| n.kind.as_deref() != Some("pack"))
            .map(|n| {
                serde_json::json!({
                    "id": n.id, "type": n.kind, "title": n.title, "name": text(&n, "name"), "description": text(&n, "description"),
                    "statement": text(&n, "statement"), "plane": n.plane,
                    "order": n.fields.get("order").and_then(|v| v.as_i64()),
                    "from": n.fields.get("from"), "to": n.fields.get("to"), "inverse": text(&n, "inverse"),
                    "template": text(&n, "template"), "stack": text(&n, "stack"), "severity": text(&n, "severity"),
                    "layer": text(&n, "plane"), "shape": text(&n, "shape"), "shapes": n.fields.get("shapes"),
                    "applies": n.fields.get("applies_to").and_then(|a| a.get("type")).and_then(|v| v.as_str()),
                })
            })
            .collect();
        out.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        out
    }
}

/// Something wrong, or worth a look, in a pack. `error` means it should not be shared as it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub pack: String,
    pub severity: &'static str,
    pub message: String,
    /// what kind of finding it is, when a library wants to treat some more strictly: `collision` or `near-name`; empty otherwise
    pub code: &'static str,
}

/// What a pack may carry: its own definitions, never ordinary nodes.
const DEFINITION_KINDS: [&str; 5] = ["type", "link-type", "rule", "plane", "derived"];

fn defined(pack: &Pack, kind: &str) -> Vec<String> {
    pack.files
        .iter()
        .filter_map(|(_, t)| parse_node(t).ok())
        .filter(|n| n.kind.as_deref() == Some(kind))
        .filter_map(|n| n.fields.get("name").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// Every pack this one brings with it or relies on, directly or not (its own name left out).
fn reach(all: &[Pack], name: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut todo = vec![name.to_string()];
    while let Some(n) = todo.pop() {
        if let Some(p) = all.iter().find(|p| p.name == n) {
            for d in p.requires.iter().map(|r| split_req(r).0.to_string()).chain(p.includes.iter().cloned()) {
                if seen.insert(d.clone()) {
                    todo.push(d);
                }
            }
        }
    }
    seen
}

/// Licences that let anyone copy the definitions into a project of their own, open or closed, with attribution at most.
const OPEN_LICENSES: [&str; 11] = [
    "Apache-2.0",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "0BSD",
    "MPL-2.0",
    "CC0-1.0",
    "CC-BY-4.0",
    "CC-BY-3.0",
    "Unlicense",
];

/// A licence expression as SPDX writes it: identifiers joined by AND, OR or WITH (and parentheses left alone).
fn spdx_like(l: &str) -> bool {
    let toks: Vec<&str> = l.split_whitespace().collect();
    !toks.is_empty()
        && toks.iter().enumerate().all(|(i, t)| {
            if i % 2 == 1 {
                ["AND", "OR", "WITH"].contains(t)
            } else {
                !t.is_empty()
                    && t.trim_matches(|c| c == '(' || c == ')')
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || ".+-:".contains(c))
            }
        })
        && toks.len() % 2 == 1
}

/// Every licence in an expression is an open one (`MIT OR Apache-2.0` is; `GPL-3.0-only` and `LicenseRef-...` are not).
fn is_open(l: &str) -> bool {
    l.split_whitespace()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .all(|(_, t)| OPEN_LICENSES.contains(&t.trim_matches(|c| c == '(' || c == ')')))
}

fn is_copyleft(l: &str) -> bool {
    l.split_whitespace().any(|t| ["GPL", "AGPL", "LGPL", "CC-BY-SA", "EUPL", "OSL"].iter().any(|c| t.starts_with(c)))
}

/// Two names a reader could take for each other: the same words with a different ending (constraint / constrains).
fn near(a: &str, b: &str) -> bool {
    if a == b || a.len() < 8 || b.len() < 8 {
        return false;
    }
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().abs_diff(b.len()) > 2 {
        return false;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i];
        for j in 1..=b.len() {
            cur.push((prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + usize::from(a[i - 1] != b[j - 1])));
        }
        prev = cur;
    }
    prev[b.len()] <= 2
}

/// Names so alike that they are easy to mix up. Types and link types are compared together, since a type and a link type that look
/// alike are the confusing case. Names inside one pack are its own business: a noun and its verb (decision / considers) is the
/// convention. The same name in two packs is not a problem: a project tells them apart with `<namespace>:<name>`.
fn lint_names(all: &[Pack]) -> Vec<Problem> {
    let mut out = Vec::new();
    for p in all {
        let names: Vec<(String, &str)> = defined(p, "type")
            .into_iter()
            .map(|n| (n, "type"))
            .chain(defined(p, "link-type").into_iter().map(|n| (n, "link type")))
            .collect();
        for q in all.iter().filter(|q| q.name > p.name) {
            let theirs: Vec<(String, &str)> = defined(q, "type")
                .into_iter()
                .map(|n| (n, "type"))
                .chain(defined(q, "link-type").into_iter().map(|n| (n, "link type")))
                .collect();
            for (a, ka) in &names {
                for (b, kb) in theirs.iter().filter(|(b, _)| near(a, b)) {
                    out.push(Problem { pack: p.name.clone(), severity: "warning", code: "near-name", message: format!("The {ka} “{a}” looks like the {kb} “{b}” in {}. Names that close get mixed up; pick a clearer word for one of them.", q.name) });
                }
            }
        }
    }
    out
}

/// The longest `use-when` that reads as one sentence in a list.
pub const USE_WHEN_MAX: usize = 200;

/// What the manifest itself must say. A `core` pack is one of the bundle every project starts from and the only kind built into the
/// binary, so only trumly may publish one. Every other pack is an `enrichment`: it adds detail to the core, says when to reach
/// for it, and needs the core (`requires: core@1`) so its definitions have something to attach to.
fn manifest_problems(all: &[Pack], p: &Pack, push: &mut impl FnMut(&'static str, String)) {
    if let Some(d) = &p.default_pattern {
        if !p.samples.iter().any(|s| s.kind == "pattern" && s.name == *d) {
            push(
                "error",
                format!("default-pattern names “{d}”, which is not one of its patterns (a folder under patterns/)."),
            );
        }
    }
    for s in &p.starter_patterns {
        if !p.samples.iter().any(|x| x.kind == "pattern" && x.name == *s) {
            push(
                "error",
                format!("starter-patterns names “{s}”, which is not one of its patterns (a folder under patterns/)."),
            );
        }
    }
    for r in &p.optional {
        let n = split_req(r).0;
        if n != p.name && !all.iter().any(|q| q.name == n) {
            push("warning", format!("optional names “{n}”, which is not a pack in the library."));
        }
    }
    match p.role.as_str() {
        "core" => {
            if p.publisher.trim() != "trumly" {
                push(
                    "error",
                    "It says role: core, which only trumly's own packs may. Use role: enrichment.".to_string(),
                );
            }
            if !p.enriches.is_empty() {
                push("warning", "A core pack does not enrich anything; leave enriches out.".to_string());
            }
        }
        "enrichment" => {
            let chars = p.use_when.trim().chars().count();
            if chars == 0 {
                push("warning", "It has no use-when. Say in one sentence when to reach for it (fields.use-when); lists and agents show it.".to_string());
            } else if chars > USE_WHEN_MAX {
                push(
                    "warning",
                    format!("use-when is {chars} characters; keep it to one sentence of at most {USE_WHEN_MAX}."),
                );
            }
            let reach_me = reach(all, &p.name);
            let core_names: Vec<&str> = all.iter().filter(|q| q.role == "core").map(|q| q.name.as_str()).collect();
            if !core_names.is_empty() && !core_names.iter().any(|c| reach_me.contains(*c)) {
                push("warning", "It does not require the core. Add requires: core@1 so its definitions have the core nodes to attach to.".to_string());
            }
            // what it enriches: a core pack, or a type that a core pack defines (a pack that adds detail to `component` says so)
            let core_types: BTreeSet<String> =
                all.iter().filter(|q| q.role == "core").flat_map(|q| defined(q, "type")).collect();
            for e in &p.enriches {
                match all.iter().find(|q| q.name == *e) {
                    Some(q) if q.role != "core" => push(
                        "warning",
                        format!(
                            "enriches names “{e}”, which is not a core pack. Name the core pack or core type it adds detail to."
                        ),
                    ),
                    Some(_) if !reach_me.contains(e.as_str()) => push(
                        "warning",
                        format!(
                            "enriches names “{e}”, but the pack does not require it. Require core@1, or the pack it enriches."
                        ),
                    ),
                    Some(_) => {}
                    None if core_types.contains(e.as_str()) => {}
                    None => push(
                        "warning",
                        format!("enriches names “{e}”, which is neither a core pack nor a type a core pack defines."),
                    ),
                }
            }
        }
        other => push("error", format!("role “{other}” is not core or enrichment.")),
    }
}

/// Check packs against each other and against what a pack is for:
/// - error: an ordinary node (not a type, link type, rule or plane) inside a pack;
/// - warning: a type or link type another pack in the library also defines, when neither brings the other (two meanings for one name);
/// - warning: a type named in a from/to list, a rule's filter or a check that neither the pack nor what it relies on defines;
/// - warning: no licence or publisher, a licence that is not an SPDX expression, a copyleft licence, or an open pack that needs a pack that is not open.
pub fn lint(all: &[Pack]) -> Vec<Problem> {
    let mut out = Vec::new();
    for p in all {
        let mut push = |severity: &'static str, message: String| {
            out.push(Problem { pack: p.name.clone(), severity, message, code: "" })
        };
        let nodes: Vec<Node> = p.files.iter().filter_map(|(_, t)| parse_node(t).ok()).collect();
        for n in nodes.iter().filter(|n| n.kind.as_deref() != Some("pack")) {
            let k = n.kind.as_deref().unwrap_or("");
            if !DEFINITION_KINDS.contains(&k) {
                push(
                    "error",
                    format!(
                        "“{}” is an ordinary node (type {}). A pack carries definitions only: types, link types, rules, planes and derived values.",
                        n.id,
                        if k.is_empty() { "none" } else { k }
                    ),
                );
            }
        }
        if p.license.trim().is_empty() {
            push("warning", "It has no licence. Say who may use it and how in fields.license: an SPDX identifier such as Apache-2.0 or CC-BY-4.0.".to_string());
        } else if !spdx_like(&p.license) {
            push(
                "warning",
                format!(
                    "The licence “{}” is not an SPDX identifier or expression (for example Apache-2.0, or LicenseRef-Acme for your own terms).",
                    p.license
                ),
            );
        } else if is_copyleft(&p.license) {
            push(
                "warning",
                format!(
                    "The licence “{}” is copyleft. A project that copies these definitions may have to be shared on the same terms; most packs should use a permissive licence.",
                    p.license
                ),
            );
        }
        if p.publisher.trim().is_empty() {
            push("warning", "It has no publisher. Say who made it and answers for it in fields.publisher.".to_string());
        }
        manifest_problems(all, p, &mut push);
        if is_open(&p.license) {
            for d in p.requires.iter().map(|r| split_req(r).0.to_string()).chain(p.includes.iter().cloned()) {
                if let Some(q) =
                    all.iter().find(|q| q.name == d).filter(|q| !q.license.trim().is_empty() && !is_open(&q.license))
                {
                    push(
                        "warning",
                        format!(
                            "It is open ({}) but needs {}, which is {}. Nobody can use this pack without accepting those terms.",
                            p.license, q.name, q.license
                        ),
                    );
                }
            }
        }
        let mine = reach(all, &p.name);
        let mut known: BTreeSet<String> = defined(p, "type").into_iter().collect();
        let works_with: BTreeSet<String> = p.optional.iter().map(|r| split_req(r).0.to_string()).collect();
        let mut known_links: BTreeSet<String> = defined(p, "link-type").into_iter().collect();
        for q in all.iter().filter(|q| mine.contains(&q.name) || works_with.contains(&q.name)) {
            known.extend(defined(q, "type"));
            known_links.extend(defined(q, "link-type"));
        }
        // `<namespace>:<name>` is known when the pack with that namespace defines the name
        let qualified = |t: &str| {
            t.split_once(':').is_some_and(|(ns, n)| {
                all.iter().any(|q| q.namespace == ns && defined(q, "type").iter().any(|d| d == n))
            })
        };
        let mut seen = BTreeSet::new();
        let mut unknown_links: Vec<(String, String)> = Vec::new();
        let mut missing = |what: &str, t: &str| {
            if !t.is_empty() && !known.contains(t) && !qualified(t) && seen.insert(format!("{what}/{t}")) {
                push(
                    "warning",
                    format!("{what} names the type “{t}”, which neither this pack nor the packs it relies on define."),
                );
            }
        };
        for n in &nodes {
            let name = n.fields.get("name").and_then(Value::as_str).unwrap_or(&n.id).to_string();
            match n.kind.as_deref() {
                Some("link-type") => {
                    for key in ["from", "to"] {
                        for t in
                            n.fields.get(key).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
                        {
                            missing(&format!("The link type “{name}”"), t);
                        }
                    }
                }
                Some("rule") => {
                    let what =
                        format!("The rule “{}”", n.fields.get("name").and_then(Value::as_str).unwrap_or(&n.title));
                    if let Some(t) = n.fields.get("applies_to").and_then(|a| a.get("type")).and_then(Value::as_str) {
                        missing(&what, t);
                    }
                    fn types_in(v: &Value, out: &mut Vec<String>) {
                        match v {
                            Value::Array(a) => a.iter().for_each(|x| types_in(x, out)),
                            Value::Object(o) => o.iter().for_each(|(k, x)| match (k.as_str(), x.as_str()) {
                                ("node_type" | "to_type" | "ancestor_type", Some(t)) => out.push(t.to_string()),
                                _ => types_in(x, out),
                            }),
                            _ => {}
                        }
                    }
                    let mut ts = Vec::new();
                    if let Some(c) = n.fields.get("check") {
                        types_in(c, &mut ts);
                    }
                    for t in ts {
                        missing(&what, &t);
                    }
                }
                Some("derived") => {
                    let what = format!("The derived value “{name}”");
                    if let Some(t) = n.fields.get("applies_to").and_then(|a| a.get("type")).and_then(Value::as_str) {
                        missing(&what, t);
                    }
                    // the links a template follows: `{runs-in/code}`, `{^belongs-to/runs-in/value}`; the last part is a field
                    let template = n.fields.get("template").and_then(Value::as_str).unwrap_or("");
                    for l in template_links(template) {
                        let known = known_links.contains(&l)
                            || l.split_once(':').is_some_and(|(ns, b)| {
                                all.iter().any(|q| q.namespace == ns && defined(q, "link-type").iter().any(|d| d == b))
                            });
                        if !known && !unknown_links.contains(&(what.clone(), l.clone())) {
                            unknown_links.push((what.clone(), l));
                        }
                    }
                }
                _ => {}
            }
        }
        for (what, l) in unknown_links {
            push(
                "warning",
                format!("{what} follows the link “{l}”, which neither this pack nor the packs it relies on define."),
            );
        }
    }
    out.extend(lint_names(all));
    out
}

/// The link names a derived template follows: every part of a `{a/b/field}` placeholder but the last, without a leading `^`;
/// `@type` is not a link.
fn template_links(template: &str) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in template.split('{').skip(1) {
        let Some((inner, _)) = chunk.split_once('}') else { continue };
        let parts: Vec<&str> = inner.split('/').collect();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            let l = part.trim_start_matches('^');
            if !l.is_empty() && l != "@type" && !out.iter().any(|o| o == l) {
                out.push(l.to_string());
            }
        }
    }
    out
}

/// What to do when a pack defines a type or link type whose name the project already defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Clash {
    /// stop and write nothing (the default)
    #[default]
    Refuse,
    /// the definition already there stays; the pack leaves its own out
    Skip,
    /// the pack's definition takes over; the one already there is deleted
    Replace,
}

#[derive(Debug, Default)]
pub struct Added {
    /// pack definitions left out because the project already defined the name (`Clash::Skip`)
    pub skipped: Vec<String>,
    /// definitions already in the project that the pack replaced (`Clash::Replace`)
    pub replaced: Vec<String>,
    pub added: Vec<String>,
    pub updated: Vec<String>,
    /// ids from an earlier version of this pack that the new version no longer has and you had changed (left in place)
    pub no_longer_in_pack: Vec<String>,
    /// ids you had changed that the upgrade left alone
    pub kept: Vec<String>,
    /// ids an earlier version had that the new one dropped, and you had not touched (deleted)
    pub removed: Vec<String>,
}

// ---------- hashing: has this node been changed since the pack put it there? ----------

fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for k in keys {
                out.push_str(&Value::String(k.clone()).to_string());
                out.push(':');
                canonical(&m[k], out);
                out.push(',');
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for x in a {
                canonical(x, out);
                out.push(',');
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// FNV-1a, 64 bit, as 16 hex digits. Not secret, just stable across versions and platforms.
fn fnv(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

/// A hash of what a node says, not how its file is laid out: reformatting a file does not change it.
pub fn node_hash(n: &Node) -> String {
    // the `pack` tag records where the node came from and changes with every version, so it is not content
    let mut bare = n.clone();
    bare.fields.remove("pack");
    let mut s = String::new();
    canonical(&node_to_json(&bare), &mut s);
    fnv(&s)
}

// ---------- versions: `requires` may say `data-model@^0.2` ----------

fn parse_ver(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.trim().split('.');
    let n = |x: Option<&str>| x.and_then(|x| x.parse::<u64>().ok());
    let (a, b, c) = (n(it.next())?, n(it.next())?, n(it.next())?);
    if it.next().is_some() { None } else { Some((a, b, c)) }
}

/// A range's version may be short: `1` is `1.0.0` and `1.2` is `1.2.0`.
fn parse_range_ver(v: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = v.trim().split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    let n = |i: usize| -> Option<u64> { parts.get(i).map_or(Some(0), |x| x.parse::<u64>().ok()) };
    Some((n(0)?, n(1)?, n(2)?))
}

/// Does `version` satisfy `range`? A range is one or more space-separated comparators, all of which must hold:
/// `^1.2.3` (compatible: below the next major, or next minor while major is 0), `~1.2.3` (below the next minor),
/// `>=`, `>`, `<=`, `<`, `=` or a bare `1.2.3` (exact). A short version is padded with zeros, so `core@1` means `core@^1.0.0`
/// and `core@1.2` means `core@^1.2.0` (a bare major or major.minor after `@` is a compatible range, not an exact one).
/// Empty or `*` accepts anything, even a version that is not x.y.z.
pub fn satisfies(version: &str, range: &str) -> bool {
    let range = range.trim();
    if range.is_empty() || range == "*" {
        return true;
    }
    let Some(v) = parse_ver(version) else { return false };
    range.split_whitespace().all(|c| {
        let (op, rest) = ["^", "~", ">=", "<=", ">", "<", "="]
            .iter()
            .find_map(|o| c.strip_prefix(o).map(|r| (*o, r)))
            .unwrap_or(("=", c));
        let Some(r) = parse_range_ver(rest) else { return false };
        let op = if op == "=" && !c.starts_with('=') && rest.split('.').count() < 3 { "^" } else { op }; // `1` and `1.2` are compatible ranges
        match op {
            "^" => {
                v >= r
                    && if r.0 > 0 {
                        v.0 == r.0
                    } else if r.1 > 0 {
                        v.0 == 0 && v.1 == r.1
                    } else {
                        v.0 == 0 && v.1 == 0 && v.2 == r.2
                    }
            }
            "~" => v >= r && v.0 == r.0 && v.1 == r.1,
            ">=" => v >= r,
            ">" => v > r,
            "<=" => v <= r,
            "<" => v < r,
            _ => v == r,
        }
    })
}

/// `data-model@^0.2` -> ("data-model", "^0.2")
fn split_req(r: &str) -> (&str, &str) {
    r.split_once('@').unwrap_or((r, ""))
}

fn md_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "md"))
        .collect();
    out.sort();
    Ok(out)
}

/// Lower-case letters and digits in words joined by single dashes: it sits before the `--` in every id.
pub fn valid_namespace(ns: &str) -> bool {
    !ns.is_empty()
        && ns.len() <= 24
        && ns.split('-').all(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
}

/// A pack's name: lowercase words joined by single hyphens. It is never used as a path, but it is checked all the same.
pub fn valid_pack_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 48
        && name.split('-').all(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
}

/// Read one pack folder and check it hangs together.
pub fn read_pack(dir: &Path) -> Result<Pack, String> {
    let mut items = Vec::new();
    for path in md_files(dir)? {
        let text = fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        items.push((path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(), text));
    }
    // examples/<name>/*.md and patterns/<name>/*.md come along with the folder they sit in as part of the file stem
    for kind in ["examples", "patterns"] {
        let base = dir.join(kind);
        if !base.is_dir() {
            continue;
        }
        let mut subs: Vec<PathBuf> = fs::read_dir(&base)
            .map_err(|e| format!("cannot read {}: {e}", base.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        subs.sort();
        for sub in subs {
            let folder = sub.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            for path in md_files(&sub)? {
                let text = fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                items.push((
                    format!("{kind}/{folder}/{}", path.file_stem().and_then(|s| s.to_str()).unwrap_or("")),
                    text,
                ));
            }
        }
    }
    pack_from_texts(&dir.display().to_string(), items)
}

/// Build a pack from (file stem, file text) pairs; `label` names it in errors. Folders and the copy in the binary both come here.
pub fn pack_from_texts(label: &str, items: Vec<(String, String)>) -> Result<Pack, String> {
    let dir = Path::new(label);
    let mut manifest = None;
    let mut files = Vec::new();
    let mut sample_items: Vec<(String, String, String)> = Vec::new();
    for (stem, text) in items {
        if let Some((kind_dir, rest)) = stem.split_once('/') {
            let (folder, file) =
                rest.split_once('/').ok_or(format!("{label}/{stem}.md: expected {kind_dir}/<name>/<file>"))?;
            sample_items.push((format!("{kind_dir}/{folder}"), file.to_string(), text));
            continue;
        }
        let node = parse_node(&text).map_err(|e| format!("{label}/{stem}.md: {}", e.as_str()))?;
        if stem != node.id {
            return Err(format!("{label}/{stem}.md: the file name must be the node id"));
        }
        if node.kind.as_deref() == Some("pack") {
            if manifest.is_some() {
                return Err(format!("{label}: more than one pack node"));
            }
            manifest = Some(node.clone());
        }
        files.push((node, text));
    }
    let m = manifest.ok_or(format!("{}: no node of type pack", dir.display()))?;
    let text = |k: &str| m.fields.get(k).and_then(|v| v.as_str()).map(str::to_string);
    let name = text("name").ok_or(format!("{}: the pack node needs fields.name", dir.display()))?;
    let version = text("version").ok_or(format!("{}: the pack node needs fields.version (a string)", dir.display()))?;
    let names = |k: &str| -> Vec<String> {
        m.fields
            .get(k)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let (requires, includes, enriches) = (names("requires"), names("includes"), names("enriches"));
    let optional = names("optional");
    let starter_patterns = names("starter-patterns");
    if !valid_pack_name(&name) {
        return Err(format!(
            "{}: the pack name '{name}' must be lowercase letters, digits and single hyphens, at most 48 characters",
            dir.display()
        ));
    }
    let namespace = text("namespace").unwrap_or_else(|| name.clone());
    if !valid_namespace(&namespace) {
        return Err(format!(
            "{}: namespace '{namespace}' must be lower-case letters, digits and single dashes",
            dir.display()
        ));
    }
    let tag = format!("{name}@{version}");
    for (n, _) in &files {
        if !n.id.starts_with(&format!("{namespace}--")) {
            return Err(format!(
                "{}: node '{}' must have an id starting '{namespace}--' (the pack's namespace)",
                dir.display(),
                n.id
            ));
        }
    }
    for (n, _) in &files {
        if n.kind.as_deref() != Some("pack") && n.fields.get("pack").and_then(|v| v.as_str()) != Some(tag.as_str()) {
            return Err(format!("{}: node '{}' must have fields.pack: {tag}", dir.display(), n.id));
        }
    }
    let samples = read_samples(label, sample_items)?;
    let title = Some(m.title.trim().to_string()).filter(|t| !t.is_empty()).unwrap_or_else(|| name.clone());
    Ok(Pack {
        title,
        description: text("description").unwrap_or_default(),
        license: text("license").unwrap_or_default(),
        publisher: text("publisher").unwrap_or_default(),
        role: text("role").unwrap_or_else(|| "enrichment".to_string()),
        enriches,
        use_when: text("use-when").unwrap_or_default(),
        default_pattern: text("default-pattern").filter(|d| !d.trim().is_empty()),
        starter_patterns,
        name,
        version,
        namespace,
        requires,
        optional,
        includes,
        files: files.into_iter().map(|(n, t)| (n.id, t)).collect(),
        samples,
    })
}

impl Sample {
    /// enough to list and choose from
    pub fn summary(&self) -> Value {
        json!({"kind": self.kind, "name": self.name, "title": self.title, "description": self.description, "use": self.usage, "requires": self.requires, "optional": self.optional, "root": self.root, "nodes": self.files.len()})
    }
    /// the summary plus every node, in the shape `get_node` and `put_node` use
    pub fn full(&self) -> Value {
        let mut v = self.summary();
        v["prefix"] = json!(self.prefix);
        v["nodes"] =
            Value::Array(self.files.iter().filter_map(|(_, t)| parse_node(t).ok()).map(|n| node_to_json(&n)).collect());
        v
    }
}

impl Pack {
    pub fn samples_summary(&self) -> Vec<Value> {
        self.samples.iter().map(Sample::summary).collect()
    }
    pub fn sample(&self, kind: &str, name: &str) -> Option<&Sample> {
        self.samples.iter().find(|s| s.kind == kind && s.name == name)
    }
    /// the pattern to lift as "another one" (`default-pattern` in the manifest), if the pack names one that it has
    pub fn default_pattern(&self) -> Option<&Sample> {
        self.default_pattern.as_deref().and_then(|d| self.sample("pattern", d))
    }
}

/// Group `(folder, file stem, text)` into samples. The folder holds a manifest (`example.md` or `pattern.md`) and its nodes.
fn read_samples(label: &str, items: Vec<(String, String, String)>) -> Result<Vec<Sample>, String> {
    let mut folders: Vec<String> = items.iter().map(|(f, _, _)| f.clone()).collect();
    folders.sort();
    folders.dedup();
    let mut out = Vec::new();
    for folder in folders {
        let (kind_dir, name) = folder.split_once('/').unwrap_or(("", &folder));
        let kind = if kind_dir == "patterns" { "pattern" } else { "example" };
        let mine: Vec<&(String, String, String)> = items.iter().filter(|(f, _, _)| *f == folder).collect();
        let manifest = mine
            .iter()
            .find(|(_, stem, _)| stem == kind)
            .ok_or(format!("{label}/{folder}: needs a {kind}.md describing it"))?;
        let m = parse_node(&manifest.2).map_err(|e| format!("{label}/{folder}/{kind}.md: {}", e.as_str()))?;
        let text = |k: &str| m.fields.get(k).and_then(|v| v.as_str()).map(str::to_string);
        let list = |k: &str| -> Vec<String> {
            m.fields
                .get(k)
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default()
        };
        let mut files = Vec::new();
        for (_, stem, body) in mine.iter().filter(|(_, stem, _)| stem != kind) {
            let n = parse_node(body).map_err(|e| format!("{label}/{folder}/{stem}.md: {}", e.as_str()))?;
            if *stem != n.id {
                return Err(format!("{label}/{folder}/{stem}.md: the file name must be the node id"));
            }
            files.push((n.id, body.clone()));
        }
        let usage = text("use").unwrap_or_else(|| "starter".to_string());
        if usage != "starter" && usage != "inspiration" {
            return Err(format!("{label}/{folder}/{kind}.md: use must be starter or inspiration"));
        }
        let root = text("root");
        if kind == "pattern" && root.as_ref().is_none_or(|r| !files.iter().any(|(id, _)| id == r)) {
            return Err(format!("{label}/{folder}/pattern.md: root must name one of the pattern's nodes"));
        }
        out.push(Sample {
            kind: kind.to_string(),
            name: name.to_string(),
            title: text("title").unwrap_or_else(|| name.to_string()),
            description: text("description").unwrap_or_default(),
            usage,
            requires: list("requires"),
            optional: list("optional"),
            root,
            prefix: text("prefix"),
            files,
        });
    }
    Ok(out)
}

/// Check that every pack shows itself in use: it has at least one example, and every example and pattern checks clean
/// (no failures, no warnings) in a project that has the packs it names. Informational findings are fine.
pub fn check_samples(all: &[Pack]) -> Vec<Problem> {
    let mut out = Vec::new();
    for p in all {
        let mut push = |severity: &'static str, message: String| {
            out.push(Problem { pack: p.name.clone(), severity, message, code: "" })
        };
        if !p.samples.iter().any(|s| s.kind == "example") {
            push("error", "It has no example. Add examples/<name>/ with an example.md and a small project that shows the pack in use.".to_string());
        }
        for s in &p.samples {
            let label = format!("{} “{}”", s.kind, s.name);
            if s.files.is_empty() {
                push("error", format!("The {label} has no nodes."));
                continue;
            }
            if s.description.trim().is_empty() {
                push("warning", format!("The {label} needs a description saying what it shows."));
            }
            if p.includes.is_empty() && !s.requires.iter().any(|r| split_req(r).0 == p.name) {
                push("warning", format!("The {label} should list its own pack ({}) in requires.", p.name));
            }
            for o in &s.optional {
                if !all.iter().any(|q| q.name == split_req(o).0) {
                    push(
                        "warning",
                        format!(
                            "The {label} lists “{}” in optional, which is not a pack in the library.",
                            split_req(o).0
                        ),
                    );
                }
            }
            match run_sample(all, p, s) {
                Ok(msgs) => {
                    for m in msgs {
                        push("error", format!("The {label} does not check clean: {m}"));
                    }
                }
                Err(e) => push("error", format!("The {label} could not be built: {e}")),
            }
        }
    }
    out
}

fn run_sample(all: &[Pack], pack: &Pack, s: &Sample) -> Result<Vec<String>, String> {
    use trumly_core::rules::Status;
    // a counter as well as the process id: two checks of the same sample at once (tests, a server) must not share a folder
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let run = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // the folder name holds only numbers: a pack's name and a sample's folder come from files someone else wrote, and must not shape a path
    let dir = std::env::temp_dir().join(format!("trumly-sample-{}-{run}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        // a sample checks clean with every pack it names present; the ones it only works with are left out by whoever lifts it
        // (an optional pack the library lacks is warned about by `check_samples`, not built)
        let known = |r: &&String| all.iter().any(|q| q.name == split_req(r).0);
        for r in s.requires.iter().chain(s.optional.iter().filter(known)) {
            let (name, _) = split_req(r);
            add_tree(all, name, None, &dir, false)?;
        }
        for (id, text) in &s.files {
            fs::write(dir.join(format!("{id}.md")), text).map_err(|e| e.to_string())?;
        }
        let report = trumly_core::report::check(crate::read_model(&dir.to_string_lossy())?);
        let mut msgs: Vec<String> = Vec::new();
        for e in &report.load_errors {
            msgs.push(format!("{}: {}", e.file, e.kind));
        }
        // title_too_long is left out for now: the shipped packs still use sentences as titles and are being reworked to short names
        // an example shows its own pack in use: a rule that matches nothing only counts when it is this pack's own (or a bundled one's),
        // so an enrichment pack's example is not asked to exercise every rule of the core it sits on
        let owned: BTreeSet<&str> =
            std::iter::once(pack.name.as_str()).chain(pack.includes.iter().map(String::as_str)).collect();
        let own_rule = |id: &str| {
            id.split_once("--")
                .and_then(|(ns, _)| all.iter().find(|q| q.namespace == ns))
                .is_some_and(|q| owned.contains(q.name.as_str()))
        };
        for f in report
            .structure
            .errors
            .iter()
            .chain(report.structure.warnings.iter())
            .filter(|f| f.kind != "title_too_long" && (f.kind != "rule_matches_nothing" || own_rule(&f.node)))
        {
            msgs.push(
                format!("{}: {} {}", f.node, f.kind, f.reference.as_deref().unwrap_or("")).trim_end().to_string(),
            );
        }
        for r in report.evaluation.results.iter().filter(|r| r.status == Status::Fail && r.severity != "info") {
            msgs.push(format!("{} fails {}", r.node, r.rule));
        }
        msgs.truncate(6);
        Ok(msgs)
    })();
    let _ = fs::remove_dir_all(&dir);
    result
}

/// Every pack folder directly inside `dir`, by name.
pub fn list_packs(dir: &Path) -> Result<Vec<Pack>, String> {
    let mut out = Vec::new();
    let mut subs: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    subs.sort();
    for s in subs {
        out.push(read_pack(&s)?);
    }
    Ok(out)
}

/// (path, id, kind, fields.pack) of every node file in a project folder.
fn model_nodes(model: &Path) -> Result<Vec<(String, String, Option<String>, Option<String>)>, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    Ok(files
        .into_iter()
        .filter_map(|(path, text)| {
            let n = parse_node(&text).ok()?;
            let pack = n.fields.get("pack").and_then(|v| v.as_str()).map(str::to_string);
            Some((path, n.id, n.kind, pack))
        })
        .collect())
}

/// The packs a project already has, as (name, version), from its `pack` nodes.
pub fn installed(model: &Path) -> Result<Vec<(String, String)>, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    let mut out: Vec<(String, String)> = files
        .into_iter()
        .filter_map(|(_, text)| {
            let n = parse_node(&text).ok().filter(|n| n.kind.as_deref() == Some("pack"))?;
            let s = |k: &str| n.fields.get(k).and_then(|v| v.as_str()).map(str::to_string);
            Some((s("name")?, s("version").unwrap_or_default()))
        })
        .collect();
    out.sort();
    Ok(out)
}

/// The packs to add for `name`, dependencies first: its `requires`, then the pack itself, then its `includes`.
/// `only` limits which of the root's children come along (a collection lets you pick); `None` takes them all.
pub fn plan<'a>(all: &'a [Pack], name: &str, only: Option<&[String]>) -> Result<Vec<&'a Pack>, String> {
    fn visit<'a>(
        all: &'a [Pack],
        name: &str,
        only: Option<&[String]>,
        out: &mut Vec<&'a Pack>,
        stack: &mut Vec<String>,
    ) -> Result<(), String> {
        if out.iter().any(|p| p.name == name) {
            return Ok(());
        }
        if stack.iter().any(|s| s == name) {
            return Err(format!("packs depend on each other in a loop: {} -> {name}", stack.join(" -> ")));
        }
        let p = all.iter().find(|p| p.name == name).ok_or_else(|| format!("no pack called {name}"))?;
        stack.push(name.to_string());
        for r in &p.requires {
            let (rn, range) = split_req(r);
            visit(all, rn, None, out, stack)?;
            let got = all.iter().find(|q| q.name == rn).map(|q| q.version.as_str()).unwrap_or("");
            if !satisfies(got, range) {
                return Err(format!("{} needs {rn}@{range}, but the packs folder has {rn} {got}", p.name));
            }
        }
        out.push(p);
        for c in &p.includes {
            if only.is_none_or(|o| o.iter().any(|x| x == c)) {
                visit(all, c, None, out, stack)?;
            }
        }
        stack.pop();
        Ok(())
    }
    if let Some(o) = only {
        let root = all.iter().find(|p| p.name == name).ok_or_else(|| format!("no pack called {name}"))?;
        if let Some(bad) = o.iter().find(|x| !root.includes.contains(x)) {
            return Err(format!("{name} does not include {bad}"));
        }
    }
    let mut out = Vec::new();
    visit(all, name, only, &mut out, &mut Vec::new())?;
    Ok(out)
}

/// (kind, name, fields.pack) of every type and link-type definition in the project.
fn definition_names(model: &Path) -> Result<Vec<(String, String, Option<String>, String, String)>, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    Ok(files
        .into_iter()
        .filter_map(|(path, text)| {
            let n = parse_node(&text).ok()?;
            let kind = n.kind.clone().filter(|k| k == "type" || k == "link-type")?;
            let name = n.fields.get("name")?.as_str()?.to_string();
            Some((kind, name, n.fields.get("pack").and_then(|v| v.as_str()).map(str::to_string), n.id.clone(), path))
        })
        .collect())
}

/// (pack name, namespace) of every pack the project has.
fn installed_namespaces(model: &Path) -> Result<Vec<(String, String)>, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    Ok(files
        .into_iter()
        .filter_map(|(_, text)| {
            let n = parse_node(&text).ok().filter(|n| n.kind.as_deref() == Some("pack"))?;
            let name = n.fields.get("name")?.as_str()?.to_string();
            let ns = n.fields.get("namespace").and_then(|v| v.as_str()).map_or_else(|| name.clone(), str::to_string);
            Some((name, ns))
        })
        .collect())
}

/// Add `name` and everything it needs or includes to a project (see `plan`). Nothing is written unless every pack in
/// the plan can go in: requirements met (at a version that satisfies the range), no namespace shared with a different
/// pack, and no clash with something the pack does not own (unless `force`). Nodes you changed since an earlier
/// version of the pack went in are kept, not overwritten (unless `force`). Returns what happened to each pack, in the
/// order added.
pub fn add_tree(
    all: &[Pack],
    name: &str,
    only: Option<&[String]>,
    model: &Path,
    force: bool,
) -> Result<Vec<(String, Added)>, String> {
    add_tree_with(all, name, only, model, force, Clash::Refuse)
}

/// `add_tree`, with a choice for what to do when a type or link type name is already defined (see `Clash`).
pub fn add_tree_with(
    all: &[Pack],
    name: &str,
    only: Option<&[String]>,
    model: &Path,
    force: bool,
    clash: Clash,
) -> Result<Vec<(String, Added)>, String> {
    let chain = plan(all, name, only)?;
    let mut have: Vec<(String, String)> = installed(model)?;
    let spaces = installed_namespaces(model)?;
    let nodes = model_nodes(model)?;
    let defs = definition_names(model)?;
    let mut conflicts = Vec::new();
    let mut name_clashes = 0usize;
    let mut plan_skip: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut plan_drop: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for pack in &chain {
        let missing: Vec<&String> = pack
            .requires
            .iter()
            .filter(|r| {
                let (rn, range) = split_req(r);
                !have.iter().any(|(n, v)| n == rn && satisfies(v, range))
            })
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{} needs {} first",
                pack.name,
                missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        have.retain(|(n, _)| *n != pack.name);
        have.push((pack.name.clone(), pack.version.clone()));
        if let Some((other, _)) = spaces.iter().find(|(n, ns)| *ns == pack.namespace && *n != pack.name) {
            return Err(format!(
                "{} cannot be added: its namespace '{}' is already used by {other}",
                pack.name, pack.namespace
            ));
        }
        let same = |p: &Option<String>| p.as_deref().is_some_and(|p| p.split('@').next() == Some(pack.name.as_str()));
        for (id, _) in &pack.files {
            if let Some((path, _, kind, from)) = nodes.iter().find(|(_, nid, _, _)| nid == id) {
                // the manifest has no provenance of its own; the same pack's manifest is simply replaced
                let own_manifest = *id == format!("{}--pack", pack.namespace) && kind.as_deref() == Some("pack");
                if !same(from) && !own_manifest {
                    conflicts.push(format!(
                        "{id} ({path}){}",
                        from.as_deref()
                            .map(|f| format!(" is from {f}"))
                            .unwrap_or_else(|| " is already in the project".into())
                    ));
                }
            }
        }
        // a type or link type with the same name that the project defined itself would make the name ambiguous; another pack's is
        // told apart by its namespace (`k8s:service`), so it is not a clash
        for (_, text) in &pack.files {
            let Ok(n) = parse_node(text) else { continue };
            let (Some(kind), Some(dn)) = (
                n.kind.clone().filter(|k| k == "type" || k == "link-type"),
                n.fields.get("name").and_then(|v| v.as_str()),
            ) else {
                continue;
            };
            if let Some((_, _, from, eid, epath)) =
                defs.iter().find(|(k, nm, from, _, _)| *k == kind && nm == dn && from.is_none())
            {
                match clash {
                    Clash::Skip => plan_skip.entry(pack.name.clone()).or_default().push(n.id.clone()),
                    Clash::Replace => {
                        plan_drop.entry(pack.name.clone()).or_default().push((eid.clone(), epath.clone()))
                    }
                    Clash::Refuse => {
                        name_clashes += 1;
                        conflicts.push(format!(
                            "the {kind} '{dn}'{}",
                            from.as_deref()
                                .map(|f| format!(" is already defined by {f}"))
                                .unwrap_or_else(|| " is already defined in the project".into())
                        ));
                    }
                }
            }
        }
    }
    if !conflicts.is_empty() && !force {
        if name_clashes == conflicts.len() {
            return Err(format!(
                "cannot add {name}: name clash with existing definitions (use the ones already there, or replace them):\n  {}",
                conflicts.join("\n  ")
            ));
        }
        return Err(format!(
            "cannot add {name}: these already exist and are not from the pack (use --force to replace them):\n  {}",
            conflicts.join("\n  ")
        ));
    }
    let mut out = Vec::new();
    for pack in chain {
        let skip = plan_skip.remove(&pack.name).unwrap_or_default();
        let drop = plan_drop.remove(&pack.name).unwrap_or_default();
        out.push((pack.name.clone(), write_pack(pack, model, force, &skip, &drop)?));
    }
    Ok(out)
}

/// The hashes a pack's manifest in the project recorded when it was installed: node id -> hash.
fn recorded(manifest: &Node) -> BTreeMap<String, String> {
    manifest
        .fields
        .get("installed")
        .and_then(Value::as_object)
        .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string()))).collect())
        .unwrap_or_default()
}

fn write_pack(
    pack: &Pack,
    model: &Path,
    force: bool,
    skip: &[String],
    drop: &[(String, String)],
) -> Result<Added, String> {
    let current: Vec<(String, Node)> = crate::read_model(&model.to_string_lossy())?
        .into_iter()
        .filter_map(|(p, t)| parse_node(&t).ok().map(|n| (p, n)))
        .collect();
    let from_this = |n: &Node| {
        n.fields.get("pack").and_then(Value::as_str).is_some_and(|p| p.split('@').next() == Some(pack.name.as_str()))
    };
    let manifest_id = format!("{}--pack", pack.namespace);
    let old = current
        .iter()
        .find(|(_, n)| {
            n.kind.as_deref() == Some("pack")
                && n.fields.get("name").and_then(Value::as_str) == Some(pack.name.as_str())
        })
        .map(|(_, n)| recorded(n))
        .unwrap_or_default();
    let put = |rel: &str, text: &str| {
        fs::write(model.join(rel), text).map_err(|e| format!("cannot write {}: {e}", model.join(rel).display()))
    };
    let mut report = Added::default();
    for (id, path) in drop {
        fs::remove_file(model.join(path)).map_err(|e| format!("cannot remove {path}: {e}"))?;
        report.replaced.push(id.clone());
    }
    let mut installed_hashes = serde_json::Map::new();
    for (id, text) in &pack.files {
        if *id == manifest_id {
            continue;
        }
        if skip.contains(id) {
            report.skipped.push(id.clone());
            continue;
        }
        let incoming = parse_node(text).map_err(|e| format!("{id}: {}", e.as_str()))?;
        let want = node_hash(&incoming);
        installed_hashes.insert(id.clone(), json!(want));
        let Some((path, node)) = current.iter().find(|(_, n)| n.id == *id) else {
            put(&format!("{id}.md"), text)?;
            report.added.push(id.clone());
            continue;
        };
        if !from_this(node) {
            put(path, text)?; // only reachable with `force`: the clash was already reported
            report.updated.push(id.clone());
            continue;
        }
        let have = node_hash(node);
        let tag_current = node.fields.get("pack") == incoming.fields.get("pack");
        match old.get(id) {
            // you changed it since it went in: your version stays, and we say so when the pack changed it too
            Some(base) if have != *base && !force => {
                if want != *base {
                    report.kept.push(id.clone());
                }
            }
            _ if have == want && tag_current => {}
            _ => {
                put(path, text)?;
                report.updated.push(id.clone());
            }
        }
    }
    // nodes an earlier version had and this one dropped
    for (path, n) in &current {
        if from_this(n) && n.kind.as_deref() != Some("pack") && !pack.files.iter().any(|(id, _)| *id == n.id) {
            if old.get(&n.id).is_some_and(|b| *b == node_hash(n)) {
                fs::remove_file(model.join(path)).map_err(|e| format!("cannot remove {path}: {e}"))?;
                report.removed.push(n.id.clone());
            } else {
                report.no_longer_in_pack.push(n.id.clone());
            }
        }
    }
    // the manifest goes in last, carrying the hashes of what the pack provided
    let (_, mtext) =
        pack.files.iter().find(|(id, _)| *id == manifest_id).ok_or_else(|| format!("{} has no manifest", pack.name))?;
    let mut m = parse_node(mtext).map_err(|e| e.as_str().to_string())?;
    m.fields.insert("installed".to_string(), Value::Object(installed_hashes));
    let existing = current.iter().find(|(_, n)| n.id == manifest_id).map(|(p, _)| p.clone());
    put(existing.as_deref().unwrap_or(&format!("{manifest_id}.md")), &node_to_markdown(&m))?;
    if existing.is_some() {
        report.updated.push(manifest_id)
    } else {
        report.added.push(manifest_id)
    }
    Ok(report)
}

/// Copy a single pack into a project whose requirements are already in place. See `add_tree` for the checks.
pub fn add(pack: &Pack, model: &Path, force: bool) -> Result<Added, String> {
    let have = installed(model)?;
    let missing: Vec<&String> = pack
        .requires
        .iter()
        .filter(|r| {
            let (rn, range) = split_req(r);
            !have.iter().any(|(n, v)| n == rn && satisfies(v, range))
        })
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} needs {} first",
            pack.name,
            missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    let alone = Pack { requires: vec![], includes: vec![], ..pack.clone() };
    add_tree(std::slice::from_ref(&alone), &alone.name, None, model, force).map(|mut v| v.remove(0).1)
}

/// What has become of an installed pack's nodes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Status {
    pub name: String,
    pub version: String,
    /// how many nodes are exactly as the pack provided them
    pub clean: usize,
    /// nodes you changed since the pack put them there
    pub changed: Vec<String>,
    /// nodes the pack provided that are no longer in the project
    pub missing: Vec<String>,
}

/// For every pack in the project: which of its nodes you have changed or deleted. Needs no copy of the pack, because
/// the manifest remembers what it installed. A pack installed before hashes were recorded reports everything clean.
pub fn status(model: &Path) -> Result<Vec<Status>, String> {
    let current: Vec<Node> =
        crate::read_model(&model.to_string_lossy())?.into_iter().filter_map(|(_, t)| parse_node(&t).ok()).collect();
    let mut out = Vec::new();
    for m in current.iter().filter(|n| n.kind.as_deref() == Some("pack")) {
        let (Some(name), version) = (
            m.fields.get("name").and_then(Value::as_str),
            m.fields.get("version").and_then(Value::as_str).unwrap_or(""),
        ) else {
            continue;
        };
        let mut st = Status { name: name.to_string(), version: version.to_string(), ..Status::default() };
        for (id, hash) in recorded(m) {
            match current.iter().find(|n| n.id == id) {
                None => st.missing.push(id),
                Some(n) if node_hash(n) == hash => st.clean += 1,
                Some(_) => st.changed.push(id),
            }
        }
        out.push(st);
    }
    Ok(out)
}

/// The types a pack defines that nodes outside the pack still use, as (type name, how many nodes). Removing the pack
/// would leave those nodes with a type nothing defines.
pub fn types_in_use(name: &str, model: &Path) -> Result<Vec<(String, usize)>, String> {
    let nodes: Vec<Node> =
        crate::read_model(&model.to_string_lossy())?.into_iter().filter_map(|(_, t)| parse_node(&t).ok()).collect();
    let tagged =
        |n: &Node| n.fields.get("pack").and_then(Value::as_str).is_some_and(|p| p.split('@').next() == Some(name));
    let mut out = Vec::new();
    for t in nodes.iter().filter(|n| n.kind.as_deref() == Some("type") && tagged(n)) {
        let Some(tn) = t.fields.get("name").and_then(Value::as_str) else { continue };
        let used = nodes.iter().filter(|n| !tagged(n) && n.kind.as_deref() == Some(tn)).count();
        if used > 0 {
            out.push((tn.to_string(), used));
        }
    }
    out.sort();
    Ok(out)
}

/// What `remove` did.
#[derive(Debug, Default)]
pub struct Removed {
    pub removed: Vec<String>,
}

/// Take a pack out of a project: its manifest and every node it provided. Refused (nothing is deleted) while another
/// pack in the project requires it, while nodes of yours link to, sit under or are placed on its nodes, or while you
/// have changed some of its nodes; `force` deletes anyway.
pub fn remove(name: &str, model: &Path, force: bool) -> Result<Removed, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    let current: Vec<(String, Node)> =
        files.into_iter().filter_map(|(p, t)| parse_node(&t).ok().map(|n| (p, n))).collect();
    let manifest = current
        .iter()
        .find(|(_, n)| n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(Value::as_str) == Some(name))
        .ok_or_else(|| format!("{name} is not in this project"))?;
    let tagged =
        |n: &Node| n.fields.get("pack").and_then(Value::as_str).is_some_and(|p| p.split('@').next() == Some(name));
    let members: Vec<&(String, Node)> = current.iter().filter(|(_, n)| tagged(n) || n.id == manifest.1.id).collect();
    let ids: Vec<&str> = members.iter().map(|(_, n)| n.id.as_str()).collect();
    if !force {
        // gather every reason at once, so the person sees the whole picture and not just the first thing in the way
        let mut sections: Vec<String> = Vec::new();
        let needers: Vec<String> = current
            .iter()
            .filter(|(_, n)| {
                n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(Value::as_str) != Some(name)
            })
            .filter(|(_, n)| {
                n.fields
                    .get("requires")
                    .and_then(Value::as_array)
                    .is_some_and(|r| r.iter().filter_map(Value::as_str).any(|r| split_req(r).0 == name))
            })
            .filter_map(|(_, n)| n.fields.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        if !needers.is_empty() {
            sections.push(format!(
                "Used by {}.\n  They depend on its types, link types or rules. Remove {} first.",
                needers.join(", "),
                if needers.len() == 1 { "that pack" } else { "those packs" }
            ));
        }
        let mut using = Vec::new();
        for (_, n) in current.iter().filter(|(_, n)| !ids.contains(&n.id.as_str())) {
            let label = if n.title != n.id { format!("{} ({})", n.title, n.id) } else { n.id.clone() };
            let target = |id: &str| {
                current
                    .iter()
                    .find(|(_, m)| m.id == id)
                    .map(|(_, m)| if m.title != m.id { format!("{} ({})", m.title, m.id) } else { m.id.clone() })
                    .unwrap_or_else(|| id.to_string())
            };
            let hits = n
                .links
                .iter()
                .filter(|l| ids.contains(&l.to.as_str()))
                .map(|l| format!("links to {} as '{}'", target(&l.to), l.kind))
                .chain(
                    n.parent.iter().filter(|p| ids.contains(&p.as_str())).map(|p| format!("sits under {}", target(p))),
                )
                .chain(
                    n.plane
                        .iter()
                        .filter(|p| ids.contains(&p.as_str()))
                        .map(|p| format!("is on the plane {}", target(p))),
                );
            for h in hits {
                using.push(format!("{label} {h}"));
            }
        }
        if !using.is_empty() {
            let more = if using.len() > 12 { format!("\n  … and {} more", using.len() - 12) } else { String::new() };
            sections.push(format!("Your project still uses it ({}):\n  {}{more}\n  Removing the pack would leave these pointing at things that no longer exist.", using.len(), using.iter().take(12).cloned().collect::<Vec<_>>().join("\n  ")));
        }
        let typed = types_in_use(name, model)?;
        if !typed.is_empty() {
            sections.push(format!("Nodes of its types are still in the project ({}).\n  Removing the pack would leave them with a type nothing defines. Delete or retype them first.", typed.iter().map(|(t, n)| format!("{n} of type {t}")).collect::<Vec<_>>().join(", ")));
        }
        let st = status(model)?.into_iter().find(|s| s.name == name).unwrap_or_default();
        if !st.changed.is_empty() {
            let names: Vec<String> = st
                .changed
                .iter()
                .map(|id| {
                    current
                        .iter()
                        .find(|(_, m)| m.id == *id)
                        .map(|(_, m)| if m.title != m.id { format!("{} ({})", m.title, m.id) } else { m.id.clone() })
                        .unwrap_or_else(|| id.clone())
                })
                .collect();
            sections.push(format!(
                "You changed {} of its nodes:\n  {}\n  Removing the pack deletes your changes with it.",
                st.changed.len(),
                names.join("\n  ")
            ));
        }
        if !sections.is_empty() {
            return Err(format!("{name} can't be removed yet.\n\n{}", sections.join("\n\n")));
        }
    }
    let mut out = Removed::default();
    for (path, n) in members {
        fs::remove_file(model.join(path)).map_err(|e| format!("cannot remove {path}: {e}"))?;
        out.removed.push(n.id.clone());
    }
    Ok(out)
}

#[derive(Debug, Default)]
pub struct Detached {
    /// The nodes that were kept, now ordinary nodes of the project.
    pub kept: Vec<String>,
}

/// Let go of a pack but keep what it gave you: the manifest is deleted and the `pack` tag is taken off every node it
/// provided, so its types, link types, rules and planes become ordinary nodes of the project, yours to edit. Nothing
/// else changes. Refused while another pack in the project requires it (its rules would lose what they rely on);
/// `force` detaches anyway. Adding the pack again later finds those nodes already there.
pub fn detach(name: &str, model: &Path, force: bool) -> Result<Detached, String> {
    let files = crate::read_model(&model.to_string_lossy())?;
    let current: Vec<(String, Node)> =
        files.into_iter().filter_map(|(p, t)| parse_node(&t).ok().map(|n| (p, n))).collect();
    let (mpath, manifest) = current
        .iter()
        .find(|(_, n)| n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(Value::as_str) == Some(name))
        .ok_or_else(|| format!("{name} is not in this project"))?;
    let tagged =
        |n: &Node| n.fields.get("pack").and_then(Value::as_str).is_some_and(|p| p.split('@').next() == Some(name));
    if !force {
        let needers: Vec<String> = current
            .iter()
            .filter(|(_, n)| {
                n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(Value::as_str) != Some(name)
            })
            .filter(|(_, n)| {
                n.fields
                    .get("requires")
                    .and_then(Value::as_array)
                    .is_some_and(|r| r.iter().filter_map(Value::as_str).any(|r| split_req(r).0 == name))
            })
            .filter_map(|(_, n)| n.fields.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        if !needers.is_empty() {
            return Err(format!(
                "{name} can't be detached yet.\n\nUsed by {}.\n  They depend on its types, link types or rules. Detach or remove {} first.",
                needers.join(", "),
                if needers.len() == 1 { "that pack" } else { "those packs" }
            ));
        }
    }
    let mut out = Detached::default();
    for (path, n) in current.iter().filter(|(_, n)| tagged(n) && n.id != manifest.id) {
        let mut plain = n.clone();
        plain.fields.remove("pack");
        fs::write(model.join(path), node_to_markdown(&plain)).map_err(|e| format!("cannot write {path}: {e}"))?;
        out.kept.push(n.id.clone());
    }
    fs::remove_file(model.join(mpath)).map_err(|e| format!("cannot remove {mpath}: {e}"))?;
    Ok(out)
}

/// Write a pack that was made in the project out to a pack folder `<packs>/<name>/`, so other projects can add it:
/// the manifest (without the install record) and every node that carries its tag. Returns the node count.
/// The folder is checked as a pack before it replaces an earlier export.
pub fn export(model: &Path, packs: &Path, name: &str) -> Result<usize, String> {
    let current: Vec<Node> =
        crate::read_model(&model.to_string_lossy())?.into_iter().filter_map(|(_, t)| parse_node(&t).ok()).collect();
    let mut manifest = current
        .iter()
        .find(|n| n.kind.as_deref() == Some("pack") && n.fields.get("name").and_then(Value::as_str) == Some(name))
        .cloned()
        .ok_or(format!("the project has no pack called '{name}'"))?;
    manifest.fields.remove("installed");
    let version = manifest.fields.get("version").and_then(Value::as_str).ok_or("the pack needs a version")?.to_string();
    let tag = format!("{name}@{version}");
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(format!("'{name}' cannot be a folder name"));
    }
    let tmp = packs.join(format!(".export-{name}"));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    let result = (|| -> Result<usize, String> {
        let put = |n: &Node| {
            fs::write(tmp.join(format!("{}.md", n.id)), node_to_markdown(n))
                .map_err(|e| format!("cannot write {}: {e}", n.id))
        };
        put(&manifest)?;
        let mut count = 0;
        for n in current.iter().filter(|n| {
            n.kind.as_deref().is_some_and(|k| DEFINITION_KINDS.contains(&k))
                && n.fields.get("pack").and_then(Value::as_str) == Some(tag.as_str())
        }) {
            put(n)?;
            count += 1;
        }
        read_pack(&tmp).map_err(|e| format!("not a valid pack yet: {e}"))?;
        Ok(count)
    })();
    match result {
        Ok(count) => {
            let dest = packs.join(name);
            let _ = fs::remove_dir_all(&dest);
            fs::rename(&tmp, &dest).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
            Ok(count)
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&tmp);
            Err(e)
        }
    }
}

/// Where each pattern that has been lifted into the project stands. A lifted node carries `from-pattern: <pack>/<name>`;
/// for every pattern with some nodes here and some not, this says how many are here (`present`), how many of the rest
/// could be added now (`fits`), and which nodes are waiting on what (`waiting`: a link or parent that is neither in the
/// pattern nor in the project, and anything hanging from such a node).
pub fn lifted_patterns(graph: &trumly_core::model::Graph, packs: &[Pack]) -> Vec<Value> {
    let samples: Vec<(String, Vec<Node>)> = packs
        .iter()
        .flat_map(|p| {
            p.samples.iter().filter(|s| s.kind == "pattern").map(move |s| {
                (format!("{}/{}", p.name, s.name), s.files.iter().filter_map(|(_, t)| parse_node(t).ok()).collect())
            })
        })
        .collect();
    lifted(graph, &samples, &[])
}

/// What `lifted_patterns` says, narrowed to what the packs in `added` just made possible: the nodes that fit now but
/// would not have fitted had those packs' definitions (ids `<namespace>--...`) not been there. A layer someone left
/// out on purpose is not offered again just because another pack was added. `fits` counts only those nodes, and
/// `fits_ids` names them; patterns with none are left out.
pub fn lifted_patterns_after(graph: &trumly_core::model::Graph, packs: &[Pack], added: &[String]) -> Vec<Value> {
    let spaces: Vec<String> =
        added.iter().filter_map(|a| packs.iter().find(|p| &p.name == a)).map(|p| p.namespace.clone()).collect();
    if spaces.is_empty() {
        return Vec::new();
    }
    let samples: Vec<(String, Vec<Node>)> = packs
        .iter()
        .flat_map(|p| {
            p.samples.iter().filter(|s| s.kind == "pattern").map(move |s| {
                (format!("{}/{}", p.name, s.name), s.files.iter().filter_map(|(_, t)| parse_node(t).ok()).collect())
            })
        })
        .collect();
    let before = lifted(graph, &samples, &spaces);
    let ids_of = |v: &Value| -> BTreeSet<String> {
        v["fits_ids"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    for mut now in lifted(graph, &samples, &[]) {
        let had = before.iter().find(|b| b["pattern"] == now["pattern"]).map(ids_of).unwrap_or_default();
        let fresh: Vec<String> = ids_of(&now).into_iter().filter(|i| !had.contains(i)).collect();
        if !fresh.is_empty() {
            now["fits"] = json!(fresh.len());
            now["fits_ids"] = json!(fresh);
            out.push(now);
        }
    }
    out
}

/// `without`: pack namespaces whose definitions are treated as not being in the project
fn lifted(graph: &trumly_core::model::Graph, samples: &[(String, Vec<Node>)], without: &[String]) -> Vec<Value> {
    let ids: BTreeSet<&str> = graph
        .nodes()
        .map(|n| n.id.as_str())
        .filter(|id| !without.iter().any(|ns| id.strip_prefix(ns.as_str()).is_some_and(|r| r.starts_with("--"))))
        .collect();
    let mut out = Vec::new();
    for (pattern, nodes) in samples {
        let here: Vec<&Node> = graph
            .nodes()
            .filter(|n| n.fields.get("from-pattern").and_then(Value::as_str) == Some(pattern.as_str()))
            .collect();
        if here.is_empty() {
            continue;
        }
        let mine: BTreeSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        let is_here = |pid: &str| here.iter().any(|h| h.id == pid || h.id.ends_with(&format!("-{pid}")));
        let rest: Vec<&Node> = nodes.iter().filter(|n| !is_here(&n.id)).collect();
        if rest.is_empty() {
            continue;
        }
        let mut blocked: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        loop {
            let mut grew = false;
            for n in &rest {
                if blocked.contains_key(n.id.as_str()) {
                    continue;
                }
                let mut needs = BTreeSet::new();
                for t in n.links.iter().map(|l| l.to.as_str()).chain(n.parent.as_deref()) {
                    if blocked.contains_key(t) {
                        needs.extend(blocked[t].iter().cloned());
                    } else if !mine.contains(t) && !ids.contains(t) {
                        needs.insert(t.to_string());
                    }
                }
                if !needs.is_empty() {
                    blocked.insert(n.id.as_str(), needs);
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        let waiting: Vec<Value> =
            blocked.iter().map(|(id, needs)| json!({"node": id, "needs": needs.iter().collect::<Vec<_>>()})).collect();
        let mut fits_ids: Vec<&str> =
            rest.iter().map(|n| n.id.as_str()).filter(|id| !blocked.contains_key(id)).collect();
        fits_ids.sort_unstable();
        out.push(json!({
            "pattern": pattern, "present": here.len(), "total": nodes.len(),
            "fits": fits_ids.len(), "fits_ids": fits_ids, "waiting": waiting,
        }));
    }
    out
}

#[cfg(test)]
mod lifted_tests {
    use super::*;

    fn n(text: &str) -> Node {
        parse_node(text).expect("a node")
    }

    #[test]
    fn a_lifted_pattern_says_what_fits_now_and_what_is_waiting() {
        let pattern = vec![
            n("---\nid: root\ntype: t\n---\n"),
            n("---\nid: a\ntype: t\nlinks:\n  - {type: uses, to: root}\n---\n"),
            n("---\nid: b\ntype: t\nlinks:\n  - {type: uses, to: far-away}\n---\n"),
            n("---\nid: c\ntype: t\nparent: b\n---\n"),
            n("---\nid: d\ntype: t\nlinks:\n  - {type: uses, to: x-known}\n---\n"),
        ];
        let mut g = trumly_core::model::Graph::new();
        assert!(g.insert(n("---\nid: p-root\ntype: t\nfields:\n  from-pattern: pk/pat\n---\n")).is_ok());
        assert!(g.insert(n("---\nid: x-known\ntype: t\n---\n")).is_ok());
        let out = lifted(&g, &[("pk/pat".into(), pattern.clone()), ("pk/other".into(), pattern)], &[]);
        assert_eq!(out.len(), 1, "only the pattern that was lifted is listed");
        assert_eq!(out[0]["present"], 1);
        assert_eq!(out[0]["total"], 5);
        assert_eq!(out[0]["fits"], 2); // a and d
        let waiting: Vec<&str> =
            out[0]["waiting"].as_array().unwrap().iter().map(|w| w["node"].as_str().unwrap()).collect();
        assert_eq!(waiting, ["b", "c"]);
        assert_eq!(out[0]["waiting"][1]["needs"][0], "far-away");
    }

    #[test]
    fn only_what_a_new_pack_made_possible_is_offered() {
        // a needs the pack's definition `nw--thing`; d needs something that was always there; b waits on nothing here
        let pattern = vec![
            n("---\nid: root\ntype: t\n---\n"),
            n("---\nid: a\ntype: t\nlinks:\n  - {type: uses, to: nw--thing}\n---\n"),
            n("---\nid: d\ntype: t\nlinks:\n  - {type: uses, to: old-known}\n---\n"),
        ];
        let mut g = trumly_core::model::Graph::new();
        assert!(g.insert(n("---\nid: p-root\ntype: t\nfields:\n  from-pattern: pk/pat\n---\n")).is_ok());
        assert!(g.insert(n("---\nid: old-known\ntype: t\n---\n")).is_ok());
        assert!(g.insert(n("---\nid: nw--thing\ntype: t\n---\n")).is_ok());
        let samples = vec![("pk/pat".to_string(), pattern)];
        let all = lifted(&g, &samples, &[]);
        assert_eq!(all[0]["fits_ids"], serde_json::json!(["a", "d"]));
        let without = lifted(&g, &samples, &["nw".to_string()]);
        assert_eq!(without[0]["fits_ids"], serde_json::json!(["d"]), "before the pack, only d fitted");
        assert_eq!(without[0]["waiting"][0]["needs"][0], "nw--thing");
    }

    #[test]
    fn a_complete_lift_is_not_listed() {
        let pattern = vec![n("---\nid: root\ntype: t\n---\n")];
        let mut g = trumly_core::model::Graph::new();
        assert!(g.insert(n("---\nid: root\ntype: t\nfields:\n  from-pattern: pk/pat\n---\n")).is_ok());
        assert!(lifted(&g, &[("pk/pat".into(), pattern)], &[]).is_empty());
    }
}
