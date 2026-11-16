use std::fs;
use std::path::{Path, PathBuf};

use trumly_fs::pack::{add, add_tree, installed, list_packs, plan, read_pack};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trumly-pack-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn packs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/packs")
}

fn names(v: &[&trumly_fs::pack::Pack]) -> Vec<String> {
    v.iter().map(|p| p.name.clone()).collect()
}

#[test]
fn the_shipped_packs_are_well_formed() {
    let all = list_packs(&packs()).unwrap();
    let got: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        got,
        [
            "adr",
            "api",
            "business",
            "data-model",
            "design-suite",
            "dns",
            "model",
            "requirements",
            "shared",
            "strategy",
            "system-design",
            "system-design-deployment",
            "system-design-resilience",
            "system-design-runtime",
            "ux"
        ]
    );
    assert!(all.iter().filter(|p| p.name != "design-suite").all(|p| p.files.len() > 2));
    // namespaces are short and distinct
    let mut ns: Vec<&str> = all.iter().map(|p| p.namespace.as_str()).collect();
    ns.sort();
    ns.dedup();
    assert_eq!(ns.len(), all.len());
    assert!(all.iter().all(|p| p.files.iter().all(|(id, _)| id.starts_with(&format!("{}--", p.namespace)))));
}

#[test]
fn adding_copies_nodes_and_records_the_manifest() {
    let model = scratch("add");
    add(&read_pack(&packs().join("shared")).unwrap(), &model, false).unwrap();
    let pack = read_pack(&packs().join("system-design")).unwrap();
    let r = add(&pack, &model, false).unwrap();
    assert!(r.updated.is_empty() && r.added.contains(&"sd--rule-component-runs".to_string()));
    assert_eq!(
        installed(&model).unwrap(),
        vec![("shared".to_string(), "0.1.0".to_string()), ("system-design".to_string(), "0.3.0".to_string())]
    );
    // adding again is an upgrade in place, not a conflict
    let again = add(&pack, &model, false).unwrap();
    assert!(again.added.is_empty() && !again.updated.is_empty());
}

#[test]
fn a_pack_that_needs_another_is_refused_until_it_is_there() {
    let model = scratch("needs");
    let data = read_pack(&packs().join("system-design-runtime")).unwrap();
    assert!(add(&data, &model, false).unwrap_err().contains("needs"));
    add(&read_pack(&packs().join("shared")).unwrap(), &model, false).unwrap();
    add(&read_pack(&packs().join("system-design")).unwrap(), &model, false).unwrap();
    add(&data, &model, false).unwrap();
}

#[test]
fn a_node_that_is_not_from_the_pack_is_a_conflict_and_nothing_is_written() {
    let model = scratch("conflict");
    add(&read_pack(&packs().join("shared")).unwrap(), &model, false).unwrap();
    fs::write(
        model.join("sd--rule-component-runs.md"),
        "---\nid: sd--rule-component-runs\ntype: rule\nfields: { statement: mine }\n---\n",
    )
    .unwrap();
    let pack = read_pack(&packs().join("system-design")).unwrap();
    let err = add(&pack, &model, false).unwrap_err();
    assert!(err.contains("sd--rule-component-runs"), "{err}");
    assert!(!model.join("sd--type-actor.md").exists(), "a refused add writes nothing");
    add(&pack, &model, true).unwrap();
    assert!(fs::read_to_string(model.join("sd--rule-component-runs.md")).unwrap().contains("system-design@0.3.0"));
}

#[test]
fn your_own_nodes_can_never_clash_by_id_but_a_definition_name_can() {
    let model = scratch("names");
    add(&read_pack(&packs().join("shared")).unwrap(), &model, false).unwrap();
    // ordinary ids have no `--`, so they cannot equal a pack's; a type named like the pack's is the real clash
    fs::write(
        model.join("type-component.md"),
        "---\nid: type-component\ntype: type\nfields: { name: component }\n---\n",
    )
    .unwrap();
    let pack = read_pack(&packs().join("system-design")).unwrap();
    let err = add(&pack, &model, false).unwrap_err();
    assert!(err.contains("the type 'component'") && err.contains("already defined in the project"), "{err}");
    assert!(!model.join("sd--pack.md").exists());
    // forcing keeps both files; the editor prefers the user's own definition
    add(&pack, &model, true).unwrap();
    assert!(model.join("type-component.md").exists() && model.join("sd--type-component.md").exists());
}

#[test]
fn a_pack_node_without_provenance_is_rejected() {
    let dir = scratch("bad");
    fs::write(dir.join("x--pack.md"), "---\nid: x--pack\ntype: pack\nfields: { name: x, version: '1' }\n---\n")
        .unwrap();
    fs::write(dir.join("x--rule-y.md"), "---\nid: x--rule-y\ntype: rule\nfields: { statement: y }\n---\n").unwrap();
    assert!(read_pack(&dir).unwrap_err().contains("fields.pack"));
}

#[test]
fn every_id_in_a_pack_carries_its_namespace() {
    let dir = scratch("ns");
    fs::write(dir.join("x--pack.md"), "---\nid: x--pack\ntype: pack\nfields: { name: x, version: '1' }\n---\n")
        .unwrap();
    fs::write(dir.join("rule-y.md"), "---\nid: rule-y\ntype: rule\nfields: { statement: y, pack: x@1 }\n---\n")
        .unwrap();
    assert!(read_pack(&dir).unwrap_err().contains("starting 'x--'"));
    let bad = scratch("ns2");
    fs::write(
        bad.join("x--pack.md"),
        "---\nid: x--pack\ntype: pack\nfields: { name: x, version: '1', namespace: 'X--Y' }\n---\n",
    )
    .unwrap();
    assert!(read_pack(&bad).unwrap_err().contains("namespace"));
}

#[test]
fn a_collection_brings_its_children_and_what_they_need_dependencies_first() {
    let all = list_packs(&packs()).unwrap();
    let everything = names(&plan(&all, "design-suite", None).unwrap());
    assert_eq!(everything.len(), 14, "the suite, core and its twelve children: {everything:?}");
    assert_eq!(everything[0], "design-suite");
    let at = |n: &str| everything.iter().position(|x| x == n).unwrap();
    assert!(
        at("shared") < at("strategy") && at("system-design") < at("system-design-runtime"),
        "needs come first: {everything:?}"
    );
    // choose only some children: a deep module pulls in the core it needs (and what that needs), the rest stay out
    assert_eq!(
        names(&plan(&all, "design-suite", Some(&["system-design-runtime".to_string()])).unwrap()),
        ["design-suite", "shared", "system-design", "system-design-runtime"]
    );
    assert!(plan(&all, "design-suite", Some(&["nope".to_string()])).unwrap_err().contains("does not include nope"));
    assert!(plan(&all, "ghost", None).unwrap_err().contains("no pack called"));
}

#[test]
fn adding_a_collection_adds_the_chosen_parts_and_is_all_or_nothing() {
    let all = list_packs(&packs()).unwrap();
    let model = scratch("tree");
    let done = add_tree(&all, "design-suite", Some(&["adr".to_string()]), &model, false).unwrap();
    assert_eq!(done.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["design-suite", "shared", "adr"]);
    assert!(model.join("adr--type-decision.md").exists() && !model.join("dm--type-data-entity.md").exists());
    // a clash anywhere in the plan stops the whole add
    let blocked = scratch("tree2");
    fs::write(blocked.join("type-entity.md"), "---\nid: type-entity\ntype: type\nfields: { name: data-entity }\n---\n")
        .unwrap();
    let err = add_tree(&all, "design-suite", None, &blocked, false).unwrap_err();
    assert!(err.contains("the type 'data-entity'"), "{err}");
    assert_eq!(fs::read_dir(&blocked).unwrap().count(), 1, "nothing written");
}

#[test]
fn two_packs_cannot_share_a_namespace() {
    let model = scratch("share");
    add(&read_pack(&packs().join("shared")).unwrap(), &model, false).unwrap();
    fs::write(
        model.join("sd--pack.md"),
        "---\nid: sd--pack\ntype: pack\nfields: { name: other-pack, namespace: sd, version: '1' }\n---\n",
    )
    .unwrap();
    let pack = read_pack(&packs().join("system-design")).unwrap();
    assert!(add(&pack, &model, true).unwrap_err().contains("namespace 'sd' is already used by other-pack"));
}

use trumly_fs::pack::{remove, satisfies, status};

#[test]
fn version_ranges() {
    assert!(satisfies("0.2.5", "^0.2.0") && !satisfies("0.3.0", "^0.2.0") && !satisfies("0.1.9", "^0.2.0"));
    assert!(satisfies("1.9.0", "^1.2.0") && !satisfies("2.0.0", "^1.2.0") && !satisfies("1.1.0", "^1.2.0"));
    assert!(satisfies("0.0.3", "^0.0.3") && !satisfies("0.0.4", "^0.0.3"));
    assert!(satisfies("1.2.9", "~1.2.3") && !satisfies("1.3.0", "~1.2.3"));
    assert!(satisfies("1.5.0", ">=1.2.0 <2.0.0") && !satisfies("2.0.0", ">=1.2.0 <2.0.0"));
    assert!(satisfies("1.2.3", "1.2.3") && satisfies("1.2.3", "=1.2.3") && !satisfies("1.2.4", "1.2.3"));
    assert!(
        satisfies("anything", "")
            && satisfies("anything", "*")
            && !satisfies("anything", "^1.0.0")
            && !satisfies("1.0.0", "^banana")
    );
}

fn pack_dir(root: &Path, name: &str, ns: &str, version: &str, requires: &str, nodes: &[(&str, &str)]) -> PathBuf {
    let d = root.join(name);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join(format!("{ns}--pack.md")), format!("---\nid: {ns}--pack\ntype: pack\nfields:\n  name: {name}\n  namespace: {ns}\n  version: {version}\n  requires: [{requires}]\n---\n")).unwrap();
    for (id, text) in nodes {
        fs::write(
            d.join(format!("{ns}--{id}.md")),
            format!("---\nid: {ns}--{id}\ntype: rule\nfields:\n  statement: {text}\n  pack: {name}@{version}\n---\n"),
        )
        .unwrap();
    }
    d
}

#[test]
fn requires_can_name_a_version_range() {
    let root = scratch("ranges");
    pack_dir(&root, "base", "b", "0.3.0", "", &[("one", "one")]);
    pack_dir(&root, "top", "t", "1.0.0", "base@^0.2.0", &[("two", "two")]);
    let all = list_packs(&root).unwrap();
    assert!(
        add_tree(&all, "top", None, &scratch("ranges-m"), false)
            .unwrap_err()
            .contains("needs base@^0.2.0, but the packs folder has base 0.3.0")
    );
    pack_dir(&root, "top", "t", "1.0.0", "base@^0.3.0", &[("two", "two")]);
    let all = list_packs(&root).unwrap();
    let m = scratch("ranges-m2");
    assert_eq!(add_tree(&all, "top", None, &m, false).unwrap().len(), 2);
}

#[test]
fn an_upgrade_replaces_what_is_untouched_keeps_what_you_changed_and_drops_what_the_pack_dropped() {
    let root = scratch("upgrade");
    let model = scratch("upgrade-m");
    pack_dir(
        &root,
        "p",
        "p",
        "0.1.0",
        "",
        &[("a", "a1"), ("b", "b1"), ("c", "c1"), ("gone", "gone1"), ("mine", "mine1")],
    );
    add_tree(&list_packs(&root).unwrap(), "p", None, &model, false).unwrap();
    assert_eq!(status(&model).unwrap()[0].clean, 5);
    // you edit b and mine, and delete c
    for (id, v) in [("b", "b-edited"), ("mine", "mine-edited")] {
        let f = model.join(format!("p--{id}.md"));
        fs::write(&f, fs::read_to_string(&f).unwrap().replace(&format!("{id}1"), v)).unwrap();
    }
    fs::remove_file(model.join("p--c.md")).unwrap();
    let st = &status(&model).unwrap()[0];
    assert_eq!(
        (st.clean, st.changed.clone(), st.missing.clone()),
        (2, vec!["p--b".to_string(), "p--mine".to_string()], vec!["p--c".to_string()])
    );
    // version 0.2.0: a, b change; mine does not; gone is dropped; c unchanged
    fs::remove_dir_all(root.join("p")).unwrap();
    pack_dir(&root, "p", "p", "0.2.0", "", &[("a", "a2"), ("b", "b2"), ("c", "c1"), ("mine", "mine1")]);
    let done = add_tree(&list_packs(&root).unwrap(), "p", None, &model, false).unwrap();
    let r = &done[0].1;
    let text = |id: &str| fs::read_to_string(model.join(format!("p--{id}.md"))).unwrap();
    assert!(text("a").contains("a2") && text("a").contains("p@0.2.0"), "untouched nodes take the new version");
    assert!(text("b").contains("b-edited"), "your edit stays");
    assert_eq!(r.kept, ["p--b"], "kept and the pack changed it too: worth saying");
    assert!(
        text("mine").contains("mine-edited") && !r.kept.contains(&"p--mine".to_string()),
        "your edit stays quietly when the pack did not touch it"
    );
    assert!(model.join("p--c.md").exists(), "a node you deleted that the pack still has comes back");
    assert!(
        !model.join("p--gone.md").exists() && r.removed == ["p--gone"],
        "an untouched node the pack dropped is removed"
    );
    // the next status is relative to the new version
    let st = &status(&model).unwrap()[0];
    assert_eq!(st.changed, ["p--b", "p--mine"]);
    // force takes the pack's version over yours
    add_tree(&list_packs(&root).unwrap(), "p", None, &model, true).unwrap();
    assert!(text("b").contains("b2") && status(&model).unwrap()[0].changed.is_empty());
}

#[test]
fn reformatting_a_file_is_not_a_change() {
    let root = scratch("fmt");
    let model = scratch("fmt-m");
    pack_dir(&root, "p", "p", "0.1.0", "", &[("a", "a1")]);
    add_tree(&list_packs(&root).unwrap(), "p", None, &model, false).unwrap();
    let f = model.join("p--a.md");
    fs::write(&f, "---\nfields: { pack: p@0.1.0, statement: a1 }\ntype: rule\nid: p--a\n---\n\n").unwrap();
    assert!(status(&model).unwrap()[0].changed.is_empty());
}

#[test]
fn removing_a_pack_is_refused_while_it_is_needed_used_or_edited() {
    let root = scratch("rm");
    let model = scratch("rm-m");
    pack_dir(&root, "base", "b", "0.1.0", "", &[("one", "one")]);
    pack_dir(&root, "top", "t", "0.1.0", "base", &[("two", "two")]);
    add_tree(&list_packs(&root).unwrap(), "top", None, &model, false).unwrap();
    assert!(remove("base", &model, false).unwrap_err().contains("Used by top"));
    // your node links to a pack node
    fs::write(model.join("mine.md"), "---\nid: mine\ntype: thing\nlinks:\n  - type: uses\n    to: t--two\n---\n")
        .unwrap();
    assert!(remove("top", &model, false).unwrap_err().contains("links to t--two as 'uses'"));
    fs::remove_file(model.join("mine.md")).unwrap();
    // you edited one
    let f = model.join("t--two.md");
    fs::write(&f, fs::read_to_string(&f).unwrap().replace("statement: two", "statement: two-edited")).unwrap();
    assert!(remove("top", &model, false).unwrap_err().contains("You changed 1 of its nodes"));
    assert!(model.join("t--pack.md").exists(), "a refused removal deletes nothing");
    let r = remove("top", &model, true).unwrap();
    assert_eq!(r.removed.len(), 2);
    assert!(!model.join("t--pack.md").exists() && model.join("b--pack.md").exists());
    remove("base", &model, false).unwrap();
    assert!(remove("base", &model, false).unwrap_err().contains("not in this project"));
}

#[test]
fn a_pack_made_in_a_model_can_be_exported_and_added_elsewhere() {
    let model = scratch("export-model");
    let lib = scratch("export-lib");
    let other = scratch("export-other");
    fs::write(model.join("acme--pack.md"), "---\nid: acme--pack\ntype: pack\ntitle: Acme\nfields:\n  name: acme\n  namespace: acme\n  version: 0.1.0\n  installed:\n    x: y\n---\n").unwrap();
    fs::write(
        model.join("acme--type-widget.md"),
        "---\nid: acme--type-widget\ntype: type\ntitle: Widget\nfields:\n  name: widget\n  pack: acme@0.1.0\n---\n",
    )
    .unwrap();
    fs::write(model.join("my-content.md"), "---\nid: my-content\ntype: widget\ntitle: Mine\n---\n").unwrap();
    assert_eq!(trumly_fs::pack::export(&model, &lib, "acme").unwrap(), 1);
    assert!(lib.join("acme/acme--type-widget.md").exists() && !lib.join("acme/my-content.md").exists());
    assert!(!fs::read_to_string(lib.join("acme/acme--pack.md")).unwrap().contains("installed"));
    let all = list_packs(&lib).unwrap();
    add(&all[0], &other, false).unwrap();
    assert!(other.join("acme--type-widget.md").exists());
    assert!(trumly_fs::pack::export(&model, &lib, "nope").is_err());
}

#[test]
fn only_the_core_bundle_is_baked_in() {
    let baked = trumly_fs::library::builtin().unwrap();
    let names: Vec<&str> = baked.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["about", "core", "define", "design", "governance", "links", "plan"]);
    assert!(
        baked.iter().all(|p| p.role == "core" && p.publisher == "trumly"),
        "every built-in pack is a core pack from trumly"
    );
    assert!(
        baked.iter().all(|p| p.samples.iter().any(|s| s.kind == "example")),
        "every pack, core included, shows itself in use"
    );
    let core = baked.iter().find(|p| p.name == "core").unwrap();
    assert_eq!(core.includes, ["links", "about", "define", "design", "plan", "governance"]);
    // every pack the bundle brings is built in, and every built-in pack other than the bundle is brought by it
    assert!(core.includes.iter().all(|i| baked.iter().any(|p| &p.name == i)));
    assert!(baked.iter().filter(|p| p.name != "core").all(|p| core.includes.contains(&p.name)));
}

#[test]
fn a_chosen_folder_beats_the_built_in_copy_and_a_core_name_cannot_be_exported() {
    let custom = scratch("lib-custom");
    tiny_pack(&custom, "about", "abt2", "mine");
    let lib = trumly_fs::library::load(Some(&custom)).unwrap();
    assert_eq!(lib.source_of("about"), Some("custom"));
    assert_eq!(lib.source_of("links"), Some("built-in"));
    assert_eq!(lib.export_dir, custom);
    let model = scratch("lib-model");
    let err = trumly_fs::library::export(&model, &lib, "design").unwrap_err();
    assert!(err.contains("core pack"), "{err}");
}

fn tiny_pack(root: &Path, name: &str, ns: &str, type_name: &str) {
    let d = root.join(name);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join(format!("{ns}--pack.md")), format!("---\nid: {ns}--pack\ntype: pack\ntitle: {name}\nfields:\n  name: {name}\n  namespace: {ns}\n  version: 1.0.0\n---\n")).unwrap();
    fs::write(d.join(format!("{ns}--type-{type_name}.md")), format!("---\nid: {ns}--type-{type_name}\ntype: type\ntitle: {type_name}\nfields:\n  name: {type_name}\n  pack: {name}@1.0.0\n---\n")).unwrap();
}

#[test]
fn two_packs_defining_the_same_name_can_use_the_existing_one_or_replace_it() {
    use trumly_fs::pack::{Clash, add_tree_with};
    let lib = scratch("clash-lib");
    tiny_pack(&lib, "first", "fst", "service");
    tiny_pack(&lib, "second", "snd", "service");
    let all = list_packs(&lib).unwrap();

    // refused by default, with a message that says it is a name clash
    let model = scratch("clash-model");
    add_tree_with(&all, "first", None, &model, false, Clash::Refuse).unwrap();
    let err = add_tree_with(&all, "second", None, &model, false, Clash::Refuse).unwrap_err();
    assert!(err.contains("name clash") && err.contains("service"), "{err}");
    assert!(!model.join("snd--pack.md").exists(), "a refused add writes nothing");

    // use the existing definition: the second pack leaves its own out and the first stays
    let r = add_tree_with(&all, "second", None, &model, false, Clash::Skip).unwrap();
    assert_eq!(r[0].1.skipped, ["snd--type-service"]);
    assert!(
        model.join("fst--type-service.md").exists()
            && !model.join("snd--type-service.md").exists()
            && model.join("snd--pack.md").exists()
    );

    // replace: the definition already there goes, the pack's comes in
    let model = scratch("clash-model2");
    add_tree_with(&all, "first", None, &model, false, Clash::Refuse).unwrap();
    let r = add_tree_with(&all, "second", None, &model, false, Clash::Replace).unwrap();
    assert_eq!(r[0].1.replaced, ["fst--type-service"]);
    assert!(!model.join("fst--type-service.md").exists() && model.join("snd--type-service.md").exists());

    // a definition of your own counts too
    let model = scratch("clash-model3");
    fs::write(
        model.join("type-service.md"),
        "---\nid: type-service\ntype: type\ntitle: service\nfields:\n  name: service\n---\n",
    )
    .unwrap();
    assert!(add_tree_with(&all, "first", None, &model, false, Clash::Refuse).unwrap_err().contains("name clash"));
    add_tree_with(&all, "first", None, &model, false, Clash::Skip).unwrap();
    assert!(model.join("type-service.md").exists() && !model.join("fst--type-service.md").exists());
}

#[test]
fn a_pack_without_an_example_is_an_error_and_a_broken_one_is_named() {
    use trumly_fs::pack::{check_samples, pack_from_texts};
    let manifest = "---\nid: t--pack\ntype: pack\nfields:\n  name: t\n  namespace: t\n  version: '1'\n  license: MIT\n  publisher: tests\n---\n".to_string();
    let bare = pack_from_texts("t", vec![("t--pack".into(), manifest.clone())]).unwrap();
    let out = check_samples(&[bare]);
    assert!(out.iter().any(|p| p.severity == "error" && p.message.contains("no example")), "{out:?}");

    let items = vec![
        ("t--pack".to_string(), manifest),
        (
            "examples/one/example".to_string(),
            "---\nid: example\ntype: example\ntitle: One\nfields:\n  description: Shows it\n  requires: [t]\n---\n"
                .to_string(),
        ),
        (
            "examples/one/thing".to_string(),
            "---\nid: thing\ntype: unknown\ntitle: A thing\nlinks:\n- type: depends-on\n  to: nowhere\n---\n"
                .to_string(),
        ),
    ];
    let p = pack_from_texts("t", items).unwrap();
    assert_eq!(p.samples.len(), 1);
    assert_eq!(p.samples[0].usage, "starter");
    let out = check_samples(&[p]);
    assert!(out.iter().any(|p| p.message.contains("does not check clean")), "{out:?}");
}

#[test]
fn a_pattern_must_name_a_root_inside_itself() {
    use trumly_fs::pack::pack_from_texts;
    let manifest =
        "---\nid: t--pack\ntype: pack\nfields:\n  name: t\n  namespace: t\n  version: '1'\n---\n".to_string();
    let items = vec![
        ("t--pack".to_string(), manifest),
        (
            "patterns/p/pattern".to_string(),
            "---\nid: pattern\ntype: pattern\ntitle: P\nfields:\n  root: missing\n---\n".to_string(),
        ),
        ("patterns/p/a".to_string(), "---\nid: a\ntype: component\ntitle: A\n---\n".to_string()),
    ];
    let err = pack_from_texts("t", items).unwrap_err();
    assert!(err.contains("root must name"), "{err}");
}

#[test]
fn the_pin_lists_exactly_what_is_built_in() {
    let pin = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/core.pin")).unwrap();
    let mut pinned: Vec<String> =
        pin.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_string).collect();
    let mut baked: Vec<String> = trumly_fs::library::builtin().unwrap().iter().map(|p| p.tag()).collect();
    pinned.sort();
    baked.sort();
    assert_eq!(pinned, baked, "packs/core.pin and the folders under packs/ must agree; run scripts/sync-core.mjs");
}
