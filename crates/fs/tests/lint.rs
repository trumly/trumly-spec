use trumly_fs::pack::{Pack, lint, pack_from_texts};

fn pack(name: &str, ns: &str, requires: &[&str], items: &[(&str, &str)]) -> Pack {
    let tag = format!("{name}@1");
    let manifest = format!(
        "---\nid: {ns}--pack\ntype: pack\nfields:\n  name: {name}\n  namespace: {ns}\n  version: '1'\n  license: MIT\n  publisher: tests\n  use-when: for tests\n  requires: [{}]\n---\n",
        requires.join(", ")
    );
    let mut files = vec![(format!("{ns}--pack"), manifest)];
    for (id, body) in items {
        files.push((format!("{ns}--{id}"), format!("---\nid: {ns}--{id}\n{body}\n---\n").replace("PACK", &tag)));
    }
    pack_from_texts(name, files).expect("a valid pack")
}

fn t(name: &str) -> String {
    format!("type: type\nfields: {{name: {name}, pack: PACK}}")
}

#[test]
fn flags_an_ordinary_node() {
    let p = pack("a", "a", &[], &[("thing", "type: component\nfields: {pack: PACK}")]);
    let out = lint(&[p]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].severity, "error");
    assert!(out[0].message.contains("ordinary node"));
}

#[test]
fn unrelated_packs_may_define_the_same_name() {
    // spec 4a: a node says which with `<namespace>:<name>`, so two packs sharing a name is not a lint finding
    let (a, b) = (pack("a", "a", &[], &[("t", &t("table"))]), pack("b", "b", &[], &[("t", &t("table"))]));
    assert!(lint(&[a, b]).is_empty());
}

#[test]
fn related_packs_may_share_a_name() {
    let (a, b) = (pack("a", "a", &[], &[("t", &t("table"))]), pack("b", "b", &["a"], &[("t", &t("table"))]));
    assert!(lint(&[a, b]).is_empty());
}

#[test]
fn flags_a_type_nobody_defines_and_accepts_one_from_a_required_pack() {
    let rule = "type: rule\nfields: {statement: s, applies_to: {type: widget}, pack: PACK}";
    let lonely = pack("b", "b", &[], &[("r", rule)]);
    assert!(lint(&[lonely]).iter().any(|p| p.message.contains("“widget”")));
    let base = pack("a", "a", &[], &[("w", &t("widget"))]);
    let user = pack("b", "b", &["a"], &[("r", rule)]);
    assert!(lint(&[base, user]).is_empty());
}

fn with(name: &str, ns: &str, extra: &str, requires: &[&str]) -> Pack {
    let manifest = format!(
        "---\nid: {ns}--pack\ntype: pack\nfields:\n  name: {name}\n  namespace: {ns}\n  version: '1'\n  use-when: for tests\n{extra}  requires: [{}]\n---\n",
        requires.join(", ")
    );
    pack_from_texts(name, vec![(format!("{ns}--pack"), manifest)]).expect("a valid pack")
}

fn says(out: &[trumly_fs::pack::Problem], pack: &str, word: &str) -> bool {
    out.iter().any(|p| p.pack == pack && p.message.contains(word))
}

#[test]
fn asks_for_a_licence_and_a_publisher() {
    let out = lint(&[with("bare", "bare", "", &[])]);
    assert!(says(&out, "bare", "no licence") && says(&out, "bare", "no publisher"), "{out:?}");
    let ok = lint(&[with("fine", "fine", "  license: Apache-2.0\n  publisher: Acme\n", &[])]);
    assert!(ok.is_empty(), "{ok:?}");
    let expr = lint(&[with("both", "both", "  license: MIT OR Apache-2.0\n  publisher: Acme\n", &[])]);
    assert!(expr.is_empty(), "{expr:?}");
}

#[test]
fn flags_a_licence_that_is_not_spdx_and_one_that_is_copyleft() {
    let out = lint(&[
        with("odd", "odd", "  license: do what you like\n  publisher: x\n", &[]),
        with("gpl", "gpl", "  license: GPL-3.0-only\n  publisher: x\n", &[]),
        with("own", "own", "  license: LicenseRef-Acme\n  publisher: x\n", &[]),
    ]);
    assert!(says(&out, "odd", "not an SPDX"), "{out:?}");
    assert!(says(&out, "gpl", "copyleft"), "{out:?}");
    assert!(!out.iter().any(|p| p.pack == "own"), "your own terms are allowed: {out:?}");
}

#[test]
fn an_open_pack_may_not_need_one_that_is_not_open() {
    let out = lint(&[
        with("base", "base", "  license: LicenseRef-Acme\n  publisher: x\n", &[]),
        with("child", "child", "  license: Apache-2.0\n  publisher: x\n", &["base"]),
        with("closed-child", "cc", "  license: LicenseRef-Acme\n  publisher: x\n", &["base"]),
    ]);
    assert!(says(&out, "child", "needs base"), "{out:?}");
    assert!(!out.iter().any(|p| p.pack == "closed-child"), "{out:?}");
}

#[test]
fn flags_names_that_are_easy_to_mix_up_across_packs_but_not_within_one() {
    let link = "type: link-type\nfields: {name: constrains, pack: PACK}";
    let a = pack("a", "a", &[], &[("c", &t("constraint"))]);
    let b = pack("b", "b", &[], &[("l", link)]);
    let out = lint(&[a, b]);
    let near: Vec<_> = out.iter().filter(|p| p.code == "near-name").collect();
    assert_eq!(near.len(), 1, "{out:?}");
    assert!(near[0].message.contains("“constraint”") && near[0].message.contains("“constrains”"), "{near:?}");
    // a noun and its verb in one pack is the convention
    let one =
        pack("c", "c", &[], &[("t", &t("decision")), ("l", "type: link-type\nfields: {name: decisions, pack: PACK}")]);
    assert!(lint(&[one]).iter().all(|p| p.code != "near-name"));
    // the same name in unrelated packs is allowed: a project tells them apart with `<namespace>:<name>`
    let x = pack("x", "x", &[], &[("w", &t("widget"))]);
    let y = pack("y", "y", &[], &[("w", &t("widget"))]);
    assert!(lint(&[x, y]).iter().all(|p| p.code != "collision"));
}

fn manifest(name: &str, ns: &str, extra: &str) -> Pack {
    let text = format!(
        "---\nid: {ns}--pack\ntype: pack\nfields:\n  name: {name}\n  namespace: {ns}\n  version: 1.0.0\n  license: Apache-2.0\n{extra}---\n"
    );
    pack_from_texts(name, vec![(format!("{ns}--pack"), text)]).expect("a valid pack")
}

#[test]
fn a_pack_is_an_enrichment_unless_it_says_core_and_only_trumly_may_say_core() {
    let plain = manifest("plain", "plain", "  publisher: Acme\n  use-when: when you need plain\n");
    assert_eq!(plain.role, "enrichment");
    let out = lint(&[
        manifest("sneaky", "sn", "  publisher: Acme\n  role: core\n"),
        manifest("ours", "ours", "  publisher: trumly\n  role: core\n"),
    ]);
    assert!(says(&out, "sneaky", "only trumly"), "{out:?}");
    assert!(!out.iter().any(|p| p.pack == "ours"), "{out:?}");
    let odd = lint(&[manifest("odd", "odd", "  publisher: Acme\n  role: sideways\n")]);
    assert!(odd.iter().any(|p| p.severity == "error" && p.message.contains("sideways")), "{odd:?}");
}

#[test]
fn an_enrichment_pack_says_when_to_use_it_and_needs_the_core() {
    let core = manifest("core", "core", "  publisher: trumly\n  role: core\n  includes: [design]\n");
    let design = manifest("design", "dsn", "  publisher: trumly\n  role: core\n");
    let bare = manifest("bare", "bare", "  publisher: Acme\n");
    let ok = manifest(
        "ok",
        "ok",
        "  publisher: Acme\n  use-when: when you run Kubernetes\n  requires: [core@1]\n  enriches: [design]\n",
    );
    let out = lint(&[core, design, bare, ok]);
    assert!(says(&out, "bare", "no use-when") && says(&out, "bare", "does not require the core"), "{out:?}");
    assert!(!out.iter().any(|p| p.pack == "ok"), "{out:?}");
}

#[test]
fn enriches_must_name_a_core_pack_the_pack_can_reach() {
    let core = manifest("core", "core", "  publisher: trumly\n  role: core\n  includes: [design, plan]\n");
    let design = manifest("design", "dsn", "  publisher: trumly\n  role: core\n");
    let plan = manifest("plan", "pln", "  publisher: trumly\n  role: core\n");
    let other = manifest("other", "oth", "  publisher: Acme\n  use-when: x\n  requires: [core@1]\n");
    let bad = manifest(
        "bad",
        "bad",
        "  publisher: Acme\n  use-when: x\n  requires: [core@1, other]\n  enriches: [nope, other]\n",
    );
    let out = lint(&[core, design, plan, other, bad]);
    assert!(
        says(&out, "bad", "“nope”") && says(&out, "bad", "“other”") && says(&out, "bad", "not a core pack"),
        "{out:?}"
    );
    // naming a core pack the requirement does not bring is a note too, but design is reached through core
    let fine = manifest(
        "fine",
        "fine",
        "  publisher: Acme\n  use-when: x\n  requires: [core@1]\n  enriches: [design, plan]\n",
    );
    let core = manifest("core", "core", "  publisher: trumly\n  role: core\n  includes: [design, plan]\n");
    let (design, plan) = (
        manifest("design", "dsn", "  publisher: trumly\n  role: core\n"),
        manifest("plan", "pln", "  publisher: trumly\n  role: core\n"),
    );
    assert!(!lint(&[core, design, plan, fine]).iter().any(|p| p.pack == "fine"));
}

#[test]
fn use_when_is_one_short_sentence() {
    let long = format!("  publisher: Acme\n  use-when: {}\n  requires: [core@1]\n", "x".repeat(201));
    let out = lint(&[manifest("wordy", "wordy", &long)]);
    assert!(says(&out, "wordy", "201 characters"), "{out:?}");
}

#[test]
fn a_short_version_in_a_range_is_compatible_not_exact() {
    use trumly_fs::pack::satisfies;
    assert!(satisfies("1.0.0", "1") && satisfies("1.4.2", "1") && !satisfies("2.0.0", "1") && !satisfies("0.9.0", "1"));
    assert!(satisfies("1.2.9", "1.2") && satisfies("1.3.0", "1.2") && !satisfies("1.1.9", "1.2"));
    assert!(satisfies("1.4.2", "^1") && satisfies("1.0.0", "^1.0.0") && !satisfies("2.0.0", "^1"));
    assert!(
        satisfies("1.0.0", "=1.0.0")
            && !satisfies("1.0.1", "=1.0.0")
            && satisfies("1.0.0", "1.0.0")
            && !satisfies("1.0.1", "1.0.0")
    );
    assert!(satisfies("3.2.1", ">=1") && satisfies("1.5.0", ">=1.2 <2"));
}

#[test]
fn enriches_may_name_a_type_the_core_defines() {
    let man = |name: &str, ns: &str, extra: &str| {
        format!(
            "---\nid: {ns}--pack\ntype: pack\nfields:\n  name: {name}\n  namespace: {ns}\n  version: 1.0.0\n  license: Apache-2.0\n{extra}---\n"
        )
    };
    let ty = |ns: &str, name: &str, pk: &str| {
        format!("---\nid: {ns}--type-{name}\ntype: type\nfields:\n  name: {name}\n  pack: {pk}\n---\n")
    };
    let core = pack_from_texts(
        "core",
        vec![
            ("cor--pack".into(), man("core", "cor", "  publisher: trumly\n  role: core\n")),
            ("cor--type-component".into(), ty("cor", "component", "core@1.0.0")),
        ],
    )
    .unwrap();
    let mk = |enriches: &str| {
        pack_from_texts(
            "hosting",
            vec![
                (
                    "hos--pack".into(),
                    man(
                        "hosting",
                        "hos",
                        &format!("  publisher: Acme\n  use-when: x\n  requires: [core@1]\n  enriches: [{enriches}]\n"),
                    ),
                ),
                ("hos--type-host".into(), ty("hos", "host", "hosting@1.0.0")),
            ],
        )
        .unwrap()
    };
    assert!(
        !lint(&[core.clone(), mk("component")]).iter().any(|p| p.pack == "hosting" && p.message.contains("enriches"))
    );
    assert!(says(&lint(&[core, mk("gadget")]), "hosting", "neither a core pack nor a type"));
}

fn with_pattern(default: &str) -> Pack {
    let manifest = format!(
        "---\nid: w--pack\ntype: pack\nfields:\n  name: w\n  namespace: w\n  version: 1.0.0\n  license: MIT\n  publisher: Acme\n  use-when: when a thing happens many times\n  requires: [core@1]\n{default}---\n"
    );
    let items = vec![
        ("w--pack".to_string(), manifest),
        (
            "patterns/flow/pattern".to_string(),
            "---\nid: pattern\ntype: pattern\ntitle: A flow\nfields:\n  root: start\n  description: A start and an end\n---\n".to_string(),
        ),
        ("patterns/flow/start".to_string(), "---\nid: start\ntype: component\ntitle: Start\n---\n".to_string()),
    ];
    pack_from_texts("w", items).expect("a valid pack")
}

#[test]
fn a_default_pattern_must_be_one_of_the_packs_patterns() {
    let ok = with_pattern("  default-pattern: flow\n");
    assert_eq!(ok.default_pattern.as_deref(), Some("flow"));
    assert_eq!(ok.default_pattern().map(|s| s.name.as_str()), Some("flow"));
    assert!(!lint(&[ok]).iter().any(|p| p.message.contains("default-pattern")));
    let bad = with_pattern("  default-pattern: nope\n");
    assert!(says(&lint(&[bad]), "w", "default-pattern names “nope”"));
    let none = with_pattern("");
    assert!(none.default_pattern.is_none() && none.default_pattern().is_none());
}

#[test]
fn a_pack_may_ship_a_derived_value_and_its_links_and_types_are_checked() {
    let d = |template: &str| {
        format!(
            "type: derived\nfields: {{name: handle, template: '{template}', applies_to: {{type: widget}}, pack: PACK}}"
        )
    };
    // the type is the pack's own and the link is defined by it: nothing to say
    let link = "type: link-type\nfields: {name: runs-in, pack: PACK}";
    let ok = pack("a", "a", &[], &[("w", &t("widget")), ("l", link), ("d", &d("{runs-in/code}"))]);
    assert!(lint(&[ok]).is_empty());
    // a link nobody defines is a warning, and a field in the template is not mistaken for one
    let bad = pack("a", "a", &[], &[("w", &t("widget")), ("d", &d("{title}-{^lives-in/code}"))]);
    let out = lint(&[bad]);
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].severity == "warning" && out[0].message.contains("“lives-in”"), "{out:?}");
    // a type nobody defines
    let lonely = pack("b", "b", &[], &[("d", &d("{title}"))]);
    assert!(lint(&[lonely]).iter().any(|p| p.message.contains("“widget”")));
}
