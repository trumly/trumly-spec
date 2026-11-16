//! Bakes the official packs (`packs/` at the repository root) into the library, so no folder is needed to use them.
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../packs");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut out = String::from("pub static EMBEDDED: &[(&str, &[(&str, &str)])] = &[\n");
    let mut dirs: Vec<PathBuf> = fs::read_dir(&root)
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    dirs.sort();
    for d in dirs {
        let mut files: Vec<PathBuf> = fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .collect();
        files.sort();
        out.push_str(&format!("    ({:?}, &[\n", d.file_name().unwrap().to_string_lossy()));
        let mut entries: Vec<(String, PathBuf)> =
            files.into_iter().map(|f| (f.file_stem().unwrap().to_string_lossy().to_string(), f)).collect();
        // examples/<name>/*.md and patterns/<name>/*.md travel with the pack; the stem carries the folder
        for kind in ["examples", "patterns"] {
            let base = d.join(kind);
            let mut subs: Vec<PathBuf> = fs::read_dir(&base)
                .map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_dir()).collect())
                .unwrap_or_default();
            subs.sort();
            for sub in subs {
                let mut fs_: Vec<PathBuf> = fs::read_dir(&sub)
                    .unwrap()
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().is_some_and(|x| x == "md"))
                    .collect();
                fs_.sort();
                for f in fs_ {
                    entries.push((
                        format!(
                            "{kind}/{}/{}",
                            sub.file_name().unwrap().to_string_lossy(),
                            f.file_stem().unwrap().to_string_lossy()
                        ),
                        f,
                    ));
                }
            }
        }
        for (stem, f) in entries {
            let canon = f.canonicalize().unwrap();
            println!("cargo:rerun-if-changed={}", canon.display());
            out.push_str(&format!("        ({:?}, include_str!({:?})),\n", stem, canon.display().to_string()));
        }
        out.push_str("    ]),\n");
    }
    out.push_str("];\n");
    fs::write(PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded.rs"), out).unwrap();
}
