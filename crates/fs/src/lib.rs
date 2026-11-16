//! The disk adapter: reads a project folder into (path, text) pairs for trumly-core.
//! Kept apart from the core so the core never touches the filesystem.

pub mod library;
pub mod pack;
pub mod start;
pub mod telemetry;

use std::fs;
use std::path::Path;

/// Every `.md` file under `dir` at any depth, as (path relative to `dir` with `/`, text).
pub fn read_model(dir: &str) -> Result<Vec<(String, String)>, String> {
    let root = Path::new(dir);
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            let rel = path.strip_prefix(root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            out.push((rel, text));
        }
    }
    Ok(())
}

/// Write the result of a core edit (`trumly_core::ops`) into a project folder: changed nodes go where their file already is
/// (a renamed node stays in the same folder), and the files of deleted ids are removed. Returns (written, removed).
pub fn apply_edit(dir: &str, edit: &trumly_core::ops::Edit) -> Result<(usize, usize), String> {
    use std::collections::BTreeMap;
    let root = Path::new(dir);
    let mut at: BTreeMap<String, String> = BTreeMap::new();
    for (rel, text) in read_model(dir)? {
        if let Ok(n) = trumly_core::parse::parse_node(&text) {
            at.entry(n.id).or_insert(rel);
        }
    }
    let folder_of = |rel: &str| rel.rsplit_once('/').map(|(d, _)| format!("{d}/")).unwrap_or_default();
    // a renamed node's new file goes beside the one it replaces
    for n in &edit.write {
        let rel = at.get(&n.id).cloned().unwrap_or_else(|| {
            let beside = edit
                .renamed
                .iter()
                .find(|(_, to)| *to == n.id)
                .and_then(|(from, _)| at.get(from))
                .map(|p| folder_of(p))
                .unwrap_or_default();
            format!("{beside}{}.md", n.id)
        });
        let path = root.join(&rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        fs::write(&path, trumly_core::format::node_to_markdown(n))
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    let mut removed = 0;
    for id in &edit.delete {
        if let Some(rel) = at.get(id) {
            if !edit.write.iter().any(|n| at.get(&n.id) == Some(rel)) {
                fs::remove_file(root.join(rel)).map_err(|e| format!("cannot remove {rel}: {e}"))?;
                removed += 1;
            }
        }
    }
    Ok((edit.write.len(), removed))
}

/// The relative path of the file holding node `id`, if there is one.
pub fn path_for_id(dir: &str, id: &str) -> Option<String> {
    read_model(dir)
        .ok()?
        .into_iter()
        .find(|(_, text)| trumly_core::parse::parse_node(text).is_ok_and(|n| n.id == id))
        .map(|(rel, _)| rel)
}

/// A revision for a file's text: it changes whenever the text does. Used so that a writer must have seen
/// the current version of a node before it may replace it. FNV-1a, so the same text gives the same revision everywhere.
pub fn rev_of_text(text: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}-{}", text.len())
}

/// The stricter rule for ids of files we create. A name that works on every system and cannot be
/// mistaken for another: lower-case letters, digits, `-` and `_`; starts and ends with a letter or digit;
/// at most 100 characters; not a name Windows reserves (`con`, `nul`, `com1`...); and no `--`, which only a pack's
/// nodes use as a namespace separator. Files that already exist keep whatever name they have, so older projects still open and save.
pub fn portable_id(id: &str, namespaced: bool) -> bool {
    const RESERVED: [&str; 4] = ["con", "prn", "aux", "nul"];
    let edge = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    let reserved = RESERVED.contains(&id)
        || ((id.starts_with("com") || id.starts_with("lpt")) && id.len() == 4 && id.as_bytes()[3].is_ascii_digit());
    !id.is_empty()
        && id.len() <= 100
        && id.chars().next().is_some_and(edge)
        && id.chars().last().is_some_and(edge)
        && id.chars().all(|c| edge(c) || c == '-' || c == '_')
        && !reserved
        && (namespaced || !id.contains("--"))
}

/// Write a file so that nobody ever reads half of it: write beside it, then rename over it.
pub fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let tmp = dir.join(format!(".{}.{}.tmp", path.file_name().unwrap().to_string_lossy(), std::process::id()));
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}
