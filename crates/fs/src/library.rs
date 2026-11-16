//! Where packs come from. Several places are searched and merged; the first place to have a pack's name wins:
//!
//! 1. `--packs DIR` / `$TRUMLY_PACKS` (source `custom`)
//! 2. `./packs` next to the project you run from (`project`)
//! 3. `~/.trumly/packs`, your own and exported packs (`yours`)
//! 4. `~/.trumly/community`, packs from other people, from unverified publishers (`community`)
//! 5. the core bundle baked into the binary (`built-in`), the only packs that are

use std::fs;
use std::path::{Path, PathBuf};

use crate::pack::{Pack, list_packs, pack_from_texts};

// The packs embedded by build.rs. Hidden from the docs: it is generated data, not API.
#[doc(hidden)]
mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}
use embedded::EMBEDDED;

#[derive(Debug, Clone)]
pub struct Entry {
    pub pack: Pack,
    /// `custom`, `project`, `yours`, `community` or `built-in`
    pub source: &'static str,
}

#[derive(Debug)]
pub struct Library {
    pub entries: Vec<Entry>,
    /// where an export goes: the first folder above that can be written to
    pub export_dir: PathBuf,
}

impl Library {
    pub fn packs(&self) -> Vec<Pack> {
        self.entries.iter().map(|e| e.pack.clone()).collect()
    }
    pub fn source_of(&self, name: &str) -> Option<&'static str> {
        self.entries.iter().find(|e| e.pack.name == name).map(|e| e.source)
    }
}

/// The core bundle, from the binary: the only packs built in. Anything else (enrichment packs) comes from a library folder or the registry.
pub fn builtin() -> Result<Vec<Pack>, String> {
    let all: Vec<Pack> = EMBEDDED
        .iter()
        .map(|(name, files)| {
            pack_from_texts(
                &format!("built-in/{name}"),
                files.iter().map(|(s, t)| (s.to_string(), t.to_string())).collect(),
            )
        })
        .collect::<Result<_, _>>()?;
    if let Some(p) = all.iter().find(|p| p.role != "core") {
        return Err(format!("built-in/{}: only core packs (role: core) are built into the binary", p.name));
    }
    Ok(all)
}

fn home() -> Option<PathBuf> {
    crate::telemetry::trumly_home()
}

/// Merge every place packs live. `explicit` is `--packs`; `$TRUMLY_PACKS` stands in when it is `None`.
pub fn load(explicit: Option<&Path>) -> Result<Library, String> {
    let explicit = explicit.map(Path::to_path_buf).or_else(|| std::env::var_os("TRUMLY_PACKS").map(PathBuf::from));
    let yours = home().map(|h| h.join("packs"));
    let community = home().map(|h| h.join("community"));
    let project = PathBuf::from("packs");
    let mut places: Vec<(&'static str, PathBuf)> = Vec::new();
    if let Some(e) = &explicit {
        places.push(("custom", e.clone()));
    } else {
        places.push(("project", project.clone()));
    }
    if let Some(y) = &yours {
        places.push(("yours", y.clone()));
    }
    if let Some(c) = &community {
        places.push(("community", c.clone()));
    }
    let mut entries: Vec<Entry> = Vec::new();
    for (source, dir) in &places {
        if dir.is_dir() {
            for pack in list_packs(dir)? {
                if !entries.iter().any(|e| e.pack.name == pack.name) {
                    entries.push(Entry { pack, source });
                }
            }
        }
    }
    // TRUMLY_NO_BUILTIN=1 leaves the core bundle out, to check or try a folder of packs on its own (the test suites do)
    if std::env::var_os("TRUMLY_NO_BUILTIN").is_none_or(|v| v.is_empty() || v == "0") {
        for pack in builtin()? {
            if !entries.iter().any(|e| e.pack.name == pack.name) {
                entries.push(Entry { pack, source: "built-in" });
            }
        }
    }
    entries.sort_by(|a, b| a.pack.name.cmp(&b.pack.name));
    let export_dir = match &explicit {
        Some(e) => e.clone(),
        None if project.is_dir() => project,
        None => yours.unwrap_or_else(|| PathBuf::from("packs")),
    };
    Ok(Library { entries, export_dir })
}

/// Export a pack made in the project to the library's export folder. A name that belongs to a core pack is refused,
/// so a local copy cannot quietly stand in for it.
pub fn export(model: &Path, lib: &Library, name: &str) -> Result<(usize, PathBuf), String> {
    if builtin()?.iter().any(|p| p.name == name) {
        return Err(format!("'{name}' is the name of a core pack. Give yours a different name."));
    }
    fs::create_dir_all(&lib.export_dir).map_err(|e| format!("cannot create {}: {e}", lib.export_dir.display()))?;
    let n = crate::pack::export(model, &lib.export_dir, name)?;
    Ok((n, lib.export_dir.join(name)))
}

/// A pack's name as it may appear in usage data: core names as they are, anything else just "custom".
pub fn public_name(name: &str) -> String {
    if builtin().is_ok_and(|b| b.iter().any(|p| p.name == name)) { name.to_string() } else { "custom".to_string() }
}
