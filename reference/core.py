#!/usr/bin/env python3
"""Reference implementation of the design graph format, v0.

This file defines behaviour. Every other implementation (the Rust core included) must produce
the same `check --json` output for every case under fixtures/cases. See spec/FORMAT.md.

  core.py check [--json] ROOT     load every .md under ROOT, validate, run rules
"""
import json
import re
import sys
from pathlib import Path

import yaml

TITLE_MAX = 30                                                            # the longest name (title) a node should have, in characters
RESERVED = {"plane", "rule", "group", "type", "link-type", "pack", "derived"}          # kernel types that rules never apply to
FRONT = re.compile(r"\A---[ \t]*\n(.*?)^---[ \t]*$\n?(.*)\Z", re.S | re.M)


# ---------- helpers ----------
def strict_eq(a, b):
    """JSON equality. Booleans never equal numbers (at any depth); 1 equals 1.0; absent never equals null."""
    if isinstance(a, bool) or isinstance(b, bool):
        return isinstance(a, bool) and isinstance(b, bool) and a == b
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(strict_eq(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(strict_eq(a[k], b[k]) for k in a)
    if isinstance(a, (list, dict)) or isinstance(b, (list, dict)):
        return False
    return a == b


class _Loader(yaml.SafeLoader):
    """YAML 1.2 core schema, so that this reference and the Rust core read every file the same way.

    PyYAML implements YAML 1.1, where `on`, `off`, `yes`, `no`, `y` and `n` are booleans (and `on:` as a
    key breaks), `0755` is octal, and an ISO date is a timestamp. None of that is wanted here:
      * booleans are only true/false (any capitalisation of the first letter, or all capitals);
      * integers are decimal, 0o octal or 0x hex; an integer beyond 64 bits is read as a float;
      * dates and times stay strings.
    """


_Loader.yaml_implicit_resolvers = {}
_R = re.compile
_Loader.add_implicit_resolver("tag:yaml.org,2002:null", _R(r"^(?:~|null|Null|NULL|)$"), ["~", "n", "N", ""])
_Loader.add_implicit_resolver("tag:yaml.org,2002:bool", _R(r"^(?:true|True|TRUE|false|False|FALSE)$"), list("tTfF"))
_Loader.add_implicit_resolver("tag:yaml.org,2002:int", _R(r"^(?:[-+]?[0-9]+|0o[0-7]+|0x[0-9a-fA-F]+)$"), list("-+0123456789"))
_Loader.add_implicit_resolver(
    "tag:yaml.org,2002:float",
    _R(r"^(?:[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$"),
    list("-+0123456789."),
)


def _construct_int(loader, node):
    v = loader.construct_scalar(node)
    sign = -1 if v.startswith("-") else 1
    v = v.lstrip("+-")
    n = int(v[2:], 16) if v.startswith("0x") else int(v[2:], 8) if v.startswith("0o") else int(v, 10)
    n *= sign
    return n if -(2 ** 63) <= n < 2 ** 63 else float(n)


def _construct_float(loader, node):
    v = loader.construct_scalar(node).lower()
    if v.lstrip("+-") == ".inf":
        return float("-inf") if v.startswith("-") else float("inf")
    return float("nan") if v == ".nan" else float(v)


_Loader.add_constructor("tag:yaml.org,2002:int", _construct_int)
_Loader.add_constructor("tag:yaml.org,2002:float", _construct_float)


def jsonable(v):
    """True if v is something JSON can hold: i64 integers, finite floats, string keys only."""
    if v is None or isinstance(v, (bool, str)):
        return True
    if isinstance(v, int):
        return -(2 ** 63) <= v < 2 ** 63
    if isinstance(v, float):
        return v == v and v not in (float("inf"), float("-inf"))
    if isinstance(v, list):
        return all(jsonable(x) for x in v)
    if isinstance(v, dict):
        return all(isinstance(k, str) and jsonable(x) for k, x in v.items())
    return False


def is_str(x):
    return isinstance(x, str) and x != ""


# ---------- load ----------
def parse_node(text):
    """Return (node, error_kind). One of the two is None."""
    m = FRONT.match(text.replace("\r\n", "\n"))
    if not m:
        return None, "invalid_frontmatter"
    try:
        fm = yaml.load(m.group(1), Loader=_Loader)
    except yaml.YAMLError:
        return None, "invalid_frontmatter"
    if not isinstance(fm, dict) or not jsonable(fm):
        return None, "invalid_frontmatter"
    if "id" not in fm or fm["id"] is None:
        return None, "missing_id"
    if not is_str(fm["id"]):
        return None, "invalid_frontmatter"
    for key in ("type", "title", "parent", "plane"):
        if fm.get(key) is not None and not isinstance(fm[key], str):
            return None, "invalid_frontmatter"
    fields = fm.get("fields")
    if fields is None:
        fields = {}
    links = fm.get("links")
    if links is None:
        links = []
    if not isinstance(fields, dict) or not isinstance(links, list):
        return None, "invalid_frontmatter"
    clean = []
    for l in links:
        if not isinstance(l, dict) or not is_str(l.get("type")) or not is_str(l.get("to")):
            return None, "invalid_frontmatter"
        if l.get("key") is not None and not isinstance(l["key"], str):
            return None, "invalid_frontmatter"
        if l.get("fields") is not None and not isinstance(l["fields"], dict):
            return None, "invalid_frontmatter"
        link = {"type": l["type"], "to": l["to"]}
        if l.get("key"):
            link["key"] = l["key"]
        if l.get("fields"):
            link["fields"] = l["fields"]
        clean.append(link)
    node = {"id": fm["id"], "type": fm.get("type"), "title": fm.get("title") or fm["id"],
            "parent": fm.get("parent"), "plane": fm.get("plane"), "fields": fields, "links": clean,
            "body": m.group(2).strip()}
    return node, None


def ignored(rel):
    """Notes that live next to a project: any README.md, and anything whose name or folder starts with a dot."""
    parts = rel.split("/")
    return parts[-1] == "README.md" or any(x.startswith(".") for x in parts)


def load(root):
    """Returns (nodes by id, load_errors)."""
    root = Path(root)
    nodes, errors = {}, []
    files = sorted((p for p in root.rglob("*.md") if p.is_file() and not ignored(p.relative_to(root).as_posix())),
                   key=lambda p: p.relative_to(root).as_posix())
    for p in files:
        rel = p.relative_to(root).as_posix()
        node, err = parse_node(p.read_text(encoding="utf-8"))
        if not err and node["id"] != p.stem:
            err = "id_filename_mismatch"
        if not err and node["id"] in nodes:
            err = "duplicate_id"
        if err:
            errors.append({"file": rel, "kind": err})
        else:
            nodes[node["id"]] = node
    return nodes, errors


# ---------- names (type and link-type resolution) ----------
# A type or link name is bare (`service`) or qualified by a pack namespace (`k8s:service`). A definition belongs to the namespace in
# its id (`k8s--type-service`); one without `--` belongs to none. `_CTX` is the namespace of the pack whose rule, derived value or
# link-type definition is being read (None for a node, a link, or a definition that is not in a pack).
_NAMES = {"nodes": None, "idx": None}
_CTX = [None]


def def_ns(d):
    return d["id"].split("--", 1)[0] if "--" in d["id"] else None


def _index(nodes):
    if _NAMES["nodes"] is not nodes:
        idx = {}
        for nid in sorted(nodes):
            d = nodes[nid]
            if d["type"] in ("type", "link-type") and is_str(d["fields"].get("name")):
                idx.setdefault((d["type"], d["fields"]["name"]), []).append(d)
        _NAMES["nodes"], _NAMES["idx"] = nodes, idx
    return _NAMES["idx"]


def namespace_levels(nodes, ns):
    """Namespaces by distance from pack `ns`: [{ns}, the packs it requires or works with, the packs those require or work with, ...].
    A pack `optional` counts only when the project has it, and then at the same level as a requirement. A namespace appears once,
    at its nearest level; cycles are fine."""
    present = {p["fields"].get("name") for p in nodes.values() if p["type"] == "pack"}
    levels, seen, cur = [], set(), {ns}
    while cur:
        levels.append(cur)
        seen |= cur
        nxt = set()
        for p in nodes.values():
            if p["type"] == "pack" and def_ns(p) in cur:
                for key in ("requires", "optional"):
                    for r in p["fields"].get(key) if isinstance(p["fields"].get(key), list) else []:
                        if is_str(r) and (key == "requires" or r.split("@")[0] in present):
                            q = pack_namespace(nodes, r.split("@")[0])
                            if q not in seen:
                                nxt.add(q)
        cur = nxt
    return levels


def candidates(nodes, kind, name, ctx=None):
    """The definitions (`kind` is "type" or "link-type") that `name` can mean, from the viewpoint of pack namespace `ctx`.
    Qualified: the definitions of that bare name in that namespace. Bare: the project's definitions of the name, except that inside
    a pack the nearest level that has any wins: the pack's own, then those of the packs it requires, then of the packs they require.
    Several namespaces at the winning level (or in the whole project) leave the name ambiguous."""
    if not is_str(name):
        return []
    idx = _index(nodes)
    if ":" in name:
        ns, bare = name.split(":", 1)
        return [d for d in idx.get((kind, bare), []) if def_ns(d) == ns]
    every = idx.get((kind, name), [])
    if ctx is not None and len(every) > 1:
        for level in namespace_levels(nodes, ctx):   # nearest first: the pack itself, what it requires, what those require
            near = [d for d in every if def_ns(d) in level]
            if near:
                return near
    return every


def same_name(nodes, kind, actual, want):
    """Does what a node or link says (`actual`) denote what a rule, link-type or template says (`want`, read from namespace _CTX)?
    By definition when both resolve; otherwise (an undefined name, or not a string) by plain equality."""
    a = candidates(nodes, kind, actual)
    w = candidates(nodes, kind, want, _CTX[0])
    if a and w:
        return bool({d["id"] for d in a} & {d["id"] for d in w})
    return strict_eq(actual, want)


def ambiguous(nodes, kind, name, ctx=None):
    """True when a bare name, read from namespace `ctx`, still has definitions in two or more pack namespaces."""
    return is_str(name) and ":" not in name and len({def_ns(d) for d in candidates(nodes, kind, name, ctx)}) > 1


def names_in(nodes, n):
    """The (kind, name) pairs a rule, derived or link-type node uses: type names and link names."""
    out = []
    f = n["fields"]

    def add(kind, v):
        for x in v if isinstance(v, list) else [v]:
            if is_str(x):
                out.append((kind, x))

    def check_names(check):
        if not isinstance(check, dict):
            return
        for key, val in check.items():
            if key == "acyclic":
                add("link-type", val)
            elif key == "any_of" and isinstance(val, list):
                for it in val:
                    check_names(it)
            elif key in ("require_link", "forbid_link", "require_link_fields", "path"):
                for it in val if isinstance(val, list) else [val]:
                    if isinstance(it, dict):
                        add("link-type", it.get("type"))
                        for k in ("node_type", "to_type", "ancestor_type"):
                            add("type", it.get(k))

    where = f.get("applies_to") if isinstance(f.get("applies_to"), dict) else {}
    if n["type"] in ("rule", "derived"):
        add("type", where.get("type"))
    if n["type"] == "rule":
        check_names(f.get("check"))
    elif n["type"] == "derived" and isinstance(f.get("template"), str):
        parts = parse_template(f["template"]) or []
        for kind, val in parts:
            if kind == "ph":
                for hop in val.split("/")[:-1]:
                    if hop != "@type" and hop:
                        add("link-type", hop[1:] if hop.startswith("^") else hop)
    elif n["type"] == "link-type":
        add("type", f.get("from"))
        add("type", f.get("to"))
    return out


# ---------- structure ----------
def ancestors(nodes, nid):
    """The node itself, then its parents. Stops at a missing parent or a repeat."""
    chain, cur = [], nid
    while cur is not None and cur in nodes and cur not in chain:
        chain.append(cur)
        cur = nodes[cur]["parent"]
    return chain


def reaches_cycle(nodes, nid):
    seen, cur = set(), nid
    while cur is not None and cur in nodes:
        if cur in seen:
            return True
        seen.add(cur)
        cur = nodes[cur]["parent"]
    return False


def pack_namespace(nodes, name):
    """The namespace a pack's nodes use: its manifest's `namespace`, else its name."""
    for nid in sorted(nodes):
        n = nodes[nid]
        if n["type"] == "pack" and n["fields"].get("name") == name:
            ns = n["fields"].get("namespace")
            return ns if is_str(ns) and ns else name
    return name


def namespace_mismatch(nodes, n):
    """Ids with `--` belong to packs. A node must say the same thing twice: its id prefix and its `fields.pack`."""
    ns = n["id"].split("--", 1)[0] if "--" in n["id"] else None
    tag = n["fields"].get("pack") if is_str(n["fields"].get("pack")) else None
    if n["type"] == "pack":
        name = n["fields"].get("name")
        return is_str(name) and ns != pack_namespace(nodes, name)
    if ns is None and tag is None:
        return False
    if ns is None or tag is None:
        return True
    return ns != pack_namespace(nodes, tag.split("@")[0])


def scratch_of(nodes, n):
    """The scratch plane a node sits in (its plane is a plane node with fields.scratch true), else None."""
    p = n["plane"]
    pl = nodes.get(p) if p is not None else None
    return p if pl is not None and pl["type"] == "plane" and pl["fields"].get("scratch") is True else None


def duplicate_definition(nodes, n):
    """The name of a type or link-type that another node of the same kind and namespace also defines, else None."""
    if n["type"] not in ("type", "link-type"):
        return None
    name = n["fields"].get("name")
    if not is_str(name):
        return None
    for m in nodes.values():
        if m["id"] != n["id"] and m["type"] == n["type"] and m["fields"].get("name") == name and def_ns(m) == def_ns(n):
            return name
    return None


def missing_required(nodes, n):
    """Names of fields the node's type definition marks `required: true` that the node leaves unset (absent, null or "")."""
    if n["type"] in RESERVED or scratch_of(nodes, n) is not None:
        return []
    need = set()
    for m in candidates(nodes, "type", n["type"]):
        if isinstance(m["fields"].get("fields"), list):
            for d in m["fields"]["fields"]:
                if isinstance(d, dict) and d.get("required") is True and is_str(d.get("name")):
                    need.add(d["name"])
    return sorted(f for f in need if n["fields"].get(f) is None or n["fields"].get(f) == "")


def endpoint_mismatches(nodes, n):
    """`type->to` for each link whose link-type definition lists `from` / `to` node types that this link's ends are not among."""
    if n["type"] in RESERVED or scratch_of(nodes, n) is not None:
        return []
    out = []
    for l in n["links"]:
        t = nodes.get(l["to"])
        if t is None:
            continue
        frm, to = [], []
        for m in candidates(nodes, "link-type", l["type"]):
            for key, acc in (("from", frm), ("to", to)):
                v = m["fields"].get(key)
                if isinstance(v, list):
                    acc.extend((x, def_ns(m)) for x in v if is_str(x))

        def among(kind_of, pairs):
            for x, ns in pairs:
                _CTX[0] = ns
                if same_name(nodes, "type", kind_of, x):
                    return True
            return False

        try:
            bad = (frm and not among(n["type"], frm)) or (to and not among(t["type"], to))
        finally:
            _CTX[0] = None
        if bad:
            out.append(f"{l['type']}->{l['to']}")
    return out


def structure(nodes):
    planes = {n["id"] for n in nodes.values() if n["type"] == "plane"}
    errors, warnings = [], []
    for nid, n in nodes.items():
        if not n["type"]:
            errors.append({"node": nid, "kind": "missing_type", "ref": None})
        p = n["parent"]
        if p is not None and p not in nodes:
            errors.append({"node": nid, "kind": "missing_parent", "ref": p})
        elif reaches_cycle(nodes, nid):
            errors.append({"node": nid, "kind": "parent_cycle", "ref": None})
        here = scratch_of(nodes, n)
        if p is not None and p in nodes and scratch_of(nodes, nodes[p]) != here:
            errors.append({"node": nid, "kind": "scratch_boundary", "ref": f"parent->{p}"})
        for l in n["links"]:
            if l["to"] in nodes and scratch_of(nodes, nodes[l["to"]]) != here:
                errors.append({"node": nid, "kind": "scratch_boundary", "ref": f"{l['type']}->{l['to']}"})
        if namespace_mismatch(nodes, n):
            warnings.append({"node": nid, "kind": "namespace_mismatch", "ref": None})
        if n["title"] != nid and len(n["title"]) > TITLE_MAX:
            warnings.append({"node": nid, "kind": "title_too_long", "ref": str(len(n["title"]))})
        dup = duplicate_definition(nodes, n)
        if dup is not None:
            warnings.append({"node": nid, "kind": "duplicate_definition", "ref": dup})
        if n["type"] in ("rule", "derived", "link-type") and def_ns(n) is not None:
            seen_names = set()
            for kind, name in names_in(nodes, n):
                if (kind, name) not in seen_names and ambiguous(nodes, kind, name, def_ns(n)):
                    seen_names.add((kind, name))
                    warnings.append({"node": nid, "kind": "ambiguous_type" if kind == "type" else "ambiguous_link_type", "ref": name})
        if n["type"] not in RESERVED and scratch_of(nodes, n) is None:
            if ambiguous(nodes, "type", n["type"]):
                warnings.append({"node": nid, "kind": "ambiguous_type", "ref": n["type"]})
            for lt in sorted({l["type"] for l in n["links"] if ambiguous(nodes, "link-type", l["type"])}):
                warnings.append({"node": nid, "kind": "ambiguous_link_type", "ref": lt})
        for r in sorted(set(endpoint_mismatches(nodes, n))):
            warnings.append({"node": nid, "kind": "link_endpoint_mismatch", "ref": r})
        for f in missing_required(nodes, n):
            warnings.append({"node": nid, "kind": "missing_required_field", "ref": f})
        if n["plane"] is not None and n["plane"] not in planes:
            warnings.append({"node": nid, "kind": "undeclared_plane", "ref": n["plane"]})
        seen = set()
        for l in n["links"]:
            if l["to"] not in nodes:
                errors.append({"node": nid, "kind": "missing_link_target", "ref": f"{l['type']}->{l['to']}"})
            ident = (l["type"], l["to"], l.get("key", ""))
            if ident in seen:
                key = f"#{l['key']}" if l.get("key") else ""
                errors.append({"node": nid, "kind": "duplicate_link", "ref": f"{l['type']}->{l['to']}{key}"})
            seen.add(ident)
    order = lambda f: (f["node"], f["kind"], f["ref"] or "")
    return sorted(errors, key=order), sorted(warnings, key=order)


# ---------- rules ----------
def matches(nodes, node, where):
    # Keys all have to match. A list under a key matches when any one of its items does ("type: [a, b]" is a or b).
    for key, want in where.items():
        wants = want if isinstance(want, list) else [want]
        if key == "type":
            ok = any(same_name(nodes, "type", node["type"], w) for w in wants)
        elif key == "plane":
            ok = any(strict_eq(node["plane"], w) for w in wants)
        elif key == "under":
            up = ancestors(nodes, node["id"])[1:]
            ok = any(is_str(w) and w in up for w in wants)
        elif key == "id":
            ok = any(is_str(x) and x == node["id"] for x in wants)
        else:
            slots = resolve(node["fields"], key)
            ok = slots is not None and any(v is not MISSING and strict_eq(v, w) for v in slots for w in wants)
        if not ok:
            return False
    return True


KNOWN_CHECKS = ("require_fields", "field_is", "compare_fields", "require_link", "forbid_link", "require_link_fields", "path", "acyclic", "any_of")
OPS = ("==", "!=", "<", "<=", ">", ">=")
TYPES = ("string", "number", "integer", "boolean", "list", "map", "null")


def as_list(v):
    return v if isinstance(v, list) else [v]


def is_number(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def type_is(v, name):
    return {"string": isinstance(v, str), "number": is_number(v),
            "integer": isinstance(v, int) and not isinstance(v, bool), "boolean": isinstance(v, bool),
            "list": isinstance(v, list), "map": isinstance(v, dict), "null": v is None}[name]


def resolve(fields, path):
    """Slots a field path reaches (None = missing). A name that is itself a field is used as is; otherwise
    names joined by `.`, and `name[]` means each item of that list. Returns None if malformed."""
    if path in fields:
        return [fields[path]]
    if "." not in path and "[" not in path and "]" not in path:
        return [MISSING]
    slots = []
    for i, seg in enumerate(path.split(".")):
        each = seg.endswith("[]")
        name = seg[:-2] if each else seg
        if not name or "[" in name or "]" in name:
            return None
        if i == 0:
            nxt = [fields.get(name, MISSING)]
        else:
            nxt = [s[name] if isinstance(s, dict) and name in s else MISSING for s in slots]
        if each:
            items = []
            for s in nxt:
                items.extend(s) if isinstance(s, list) else items.append(MISSING)
            nxt = items
        slots = nxt
    return slots


MISSING = object()


def present(v):
    return v is not MISSING and v is not None and v != ""


def check_require_fields(node, names):
    if not isinstance(names, list) or not all(isinstance(n, str) for n in names):
        return "bad"
    ok = True
    for n in names:
        slots = resolve(node["fields"], n)
        if slots is None:
            return "bad"
        ok = ok and all(present(s) for s in slots)
    return "pass" if ok else "fail"


FIELD_IS_KEYS = {"in", "min", "max", "type", "optional", "min_items", "max_items", "unique", "min_length", "max_length", "max_bytes"}


def non_neg_int(v):
    return isinstance(v, int) and not isinstance(v, bool) and v >= 0


def unique_ok(items, by):
    keys = []
    for it in items:
        if by is True:
            keys.append(it)
        elif isinstance(it, dict):
            s = resolve(it, by)
            if s and s[0] is not MISSING:
                keys.append(s[0])
    return all(not any(strict_eq(j, k) for j in keys[:i]) for i, k in enumerate(keys))


def check_field_is(node, spec_map, calc=None):
    """`calc` is the node's calculated values by name ({name: text or list of text}); a name the node has no field of is read from there."""
    own = {"id": node["id"], "title": node["title"]}   # a node's own id and title read like fields (title is the id when the file gives none)
    fields = {**(calc or {}), **own, **node["fields"]}
    if not isinstance(spec_map, dict):
        return "bad"
    outcome = "pass"
    for name, spec in spec_map.items():
        if not isinstance(spec, dict) or set(spec) - FIELD_IS_KEYS:
            return "bad"
        if "in" in spec and not isinstance(spec["in"], list):
            return "bad"
        if any(k in spec and not is_number(spec[k]) for k in ("min", "max")):
            return "bad"
        if any(k in spec and not non_neg_int(spec[k]) for k in ("min_items", "max_items", "min_length", "max_length", "max_bytes")):
            return "bad"
        if "unique" in spec:
            u = spec["unique"]
            if not (u is True or (isinstance(u, str) and u and "[]" not in u and resolve({}, u) is not None)):
                return "bad"
        if "type" in spec and spec["type"] not in TYPES:
            return "bad"
        if name not in node["fields"] and name not in own and isinstance((calc or {}).get(name), list):
            slots = list(calc[name])   # a calculated name with several values is checked value by value
        else:
            slots = resolve(fields, name)
        if slots is None:
            return "bad"
        for v in slots:
            if v is MISSING:
                if spec.get("optional") is not True:
                    outcome = "fail"
                continue
            if "in" in spec and not any(strict_eq(v, x) for x in spec["in"]):
                outcome = "fail"
            if "min" in spec and not (is_number(v) and v >= spec["min"]):
                outcome = "fail"
            if "max" in spec and not (is_number(v) and v <= spec["max"]):
                outcome = "fail"
            if "type" in spec and not type_is(v, spec["type"]):
                outcome = "fail"
            if "min_items" in spec and not (isinstance(v, list) and len(v) >= spec["min_items"]):
                outcome = "fail"
            if "max_items" in spec and not (isinstance(v, list) and len(v) <= spec["max_items"]):
                outcome = "fail"
            if "unique" in spec and not (isinstance(v, list) and unique_ok(v, spec["unique"])):
                outcome = "fail"
            if "min_length" in spec and not (isinstance(v, str) and len(v) >= spec["min_length"]):
                outcome = "fail"
            if "max_length" in spec and not (isinstance(v, str) and len(v) <= spec["max_length"]):
                outcome = "fail"
            if "max_bytes" in spec and not (isinstance(v, str) and len(v.encode("utf-8")) <= spec["max_bytes"]):
                outcome = "fail"
    return outcome


def check_compare_fields(node, items):
    outcome = "pass"
    for it in as_list(items):
        if not isinstance(it, dict) or set(it) - {"left", "op", "right", "value"}:
            return "bad"
        if not isinstance(it.get("left"), str) or it.get("op") not in OPS or ("right" in it) == ("value" in it):
            return "bad"
        if "right" in it and not isinstance(it["right"], str):
            return "bad"
        f = node["fields"]
        lefts = resolve(f, it["left"])
        if lefts is None:
            return "bad"
        if "right" in it:
            rs = resolve(f, it["right"])
            if rs is None or len(rs) != 1 or "[]" in it["right"]:
                return "bad"
            r = rs[0]
        else:
            r = it["value"]
        op = it["op"]
        for l in lefts:
            if l is MISSING or r is MISSING:
                outcome = "fail"
                continue
            if op == "==":
                ok = strict_eq(l, r)
            elif op == "!=":
                ok = not strict_eq(l, r)
            elif (is_number(l) and is_number(r)) or (isinstance(l, str) and isinstance(r, str)):
                ok = {"<": l < r, "<=": l <= r, ">": l > r, ">=": l >= r}[op]
            else:
                ok = False
            if not ok:
                outcome = "fail"
    return outcome


def nearest_ancestor(nodes, nid, typ):
    for a in ancestors(nodes, nid)[1:]:
        if same_name(nodes, "type", nodes[a]["type"], typ):
            return a
    return None


def link_item_ok(item, keys):
    return (isinstance(item, dict) and not set(item) - set(keys) and is_str(item.get("type"))
            and item.get("dir", "out") in ("out", "in"))


def count_links(nodes, inbound, node, item):
    """Links of this node (out) or to this node (in) that satisfy every filter in `item`."""
    t = item["type"]
    if item.get("dir", "out") == "out":
        pairs = [(l, nodes.get(l["to"])) for l in node["links"] if same_name(nodes, "link-type", l["type"], t)]
    else:
        pairs = [(l, nodes[src]) for src, l in inbound.get(node["id"], []) if same_name(nodes, "link-type", l["type"], t)]
    ntype = item["node_type"] if "node_type" in item else item.get("to_type", ...)
    nplane = item["node_plane"] if "node_plane" in item else item.get("to_plane", ...)
    n = 0
    for l, other in pairs:
        if other is None:
            continue
        if ntype is not ... and not same_name(nodes, "type", other["type"], ntype):
            continue
        if nplane is not ... and not strict_eq(other["plane"], nplane):
            continue
        if "node_fields" in item and not all(k in other["fields"] and strict_eq(other["fields"][k], v) for k, v in item["node_fields"].items()):
            continue
        if "same_fields" in item and not all(f in node["fields"] and f in other["fields"] and strict_eq(node["fields"][f], other["fields"][f]) for f in item["same_fields"]):
            continue
        if "ancestor_type" in item:
            a, b = nearest_ancestor(nodes, node["id"], item["ancestor_type"]), nearest_ancestor(nodes, other["id"], item["ancestor_type"])
            if item["scope"] == "same" and not (a is not None and a == b):
                continue
            if item["scope"] == "other" and not (a is not None and b is not None and a != b):
                continue
        if "with_fields" in item and not all(present((l.get("fields") or {}).get(f)) for f in item["with_fields"]):
            continue
        n += 1
    return n


LINK_KEYS = ("type", "dir", "node_type", "node_plane", "to_type", "to_plane", "node_fields", "same_fields", "ancestor_type", "scope", "with_fields", "min", "max")


def link_bounds(item):
    lo = item.get("min", 0 if "max" in item else 1)
    return lo, item.get("max")


def link_item_valid(item):
    if not link_item_ok(item, LINK_KEYS):
        return False
    if ("ancestor_type" in item) != ("scope" in item) or item.get("scope", "same") not in ("same", "other"):
        return False
    if "ancestor_type" in item and not is_str(item["ancestor_type"]):
        return False
    if "node_fields" in item and not (isinstance(item["node_fields"], dict) and all(is_str(k) for k in item["node_fields"])):
        return False
    if "same_fields" in item and not (isinstance(item["same_fields"], list) and all(isinstance(f, str) for f in item["same_fields"])):
        return False
    if "with_fields" in item and not (isinstance(item["with_fields"], list) and all(isinstance(f, str) for f in item["with_fields"])):
        return False
    return all(k not in item or (isinstance(item[k], int) and not isinstance(item[k], bool) and item[k] >= 0) for k in ("min", "max"))


def check_require_link(nodes, inbound, node, items):
    outcome = "pass"
    for it in as_list(items):
        if not link_item_valid(it):
            return "bad"
        lo, hi = link_bounds(it)
        n = count_links(nodes, inbound, node, it)
        if n < lo or (hi is not None and n > hi):
            outcome = "fail"
    return outcome


def check_forbid_link(nodes, inbound, node, items):
    """Sugar for `max: 0`: the same filters as require_link, but min and max are not accepted."""
    outcome = "pass"
    for it in as_list(items):
        if not link_item_valid(it) or "min" in it or "max" in it:
            return "bad"
        if count_links(nodes, inbound, node, it) > 0:
            outcome = "fail"
    return outcome


STEP_KEYS = ("type", "dir", "node_type", "node_plane", "node_fields")


def step_valid(step):
    return (link_item_ok(step, STEP_KEYS) and
            ("node_fields" not in step or (isinstance(step["node_fields"], dict) and all(is_str(k) for k in step["node_fields"]))))


def step_targets(nodes, inbound, nid, step):
    """Ids at the other end of `nid`'s links of this step's type that exist and match the step's filters."""
    if step.get("dir", "out") == "out":
        ends = [l["to"] for l in nodes[nid]["links"] if same_name(nodes, "link-type", l["type"], step["type"])]
    else:
        ends = [src for src, l in inbound.get(nid, []) if same_name(nodes, "link-type", l["type"], step["type"])]
    out = set()
    for e in ends:
        o = nodes.get(e)
        if o is None:
            continue
        if "node_type" in step and not same_name(nodes, "type", o["type"], step["node_type"]):
            continue
        if "node_plane" in step and not strict_eq(o["plane"], step["node_plane"]):
            continue
        if "node_fields" in step and not all(k in o["fields"] and strict_eq(o["fields"][k], v) for k, v in step["node_fields"].items()):
            continue
        out.add(e)
    return out


def check_path(nodes, inbound, node, steps):
    """A chain of link steps must exist from this node: each step is followed from every node the last one reached."""
    if not isinstance(steps, list) or not steps or not all(isinstance(s, dict) and step_valid(s) for s in steps):
        return "bad"
    frontier = {node["id"]}
    for step in steps:
        frontier = set().union(*(step_targets(nodes, inbound, f, step) for f in frontier))
        if not frontier:
            return "fail"
    return "pass"


def check_acyclic(nodes, node, types):
    """Fails if this node can reach itself by following links of the given type(s)."""
    types = as_list(types)
    if not types or not all(is_str(t) and t for t in types):
        return "bad"
    for t in types:
        seen, stack = set(), [l["to"] for l in node["links"] if same_name(nodes, "link-type", l["type"], t)]
        while stack:
            cur = stack.pop()
            if cur == node["id"]:
                return "fail"
            if cur in seen or cur not in nodes:
                continue
            seen.add(cur)
            stack += [l["to"] for l in nodes[cur]["links"] if same_name(nodes, "link-type", l["type"], t)]
    return "pass"


def check_require_link_fields(nodes, inbound, node, items):
    outcome = "pass"
    for it in as_list(items):
        if not link_item_ok(it, ("type", "dir", "fields")) or not (isinstance(it.get("fields"), list) and all(isinstance(f, str) for f in it["fields"])):
            return "bad"
        if it.get("dir", "out") == "out":
            links = [l for l in node["links"] if same_name(nodes, "link-type", l["type"], it["type"])]
        else:
            links = [l for src, l in inbound.get(node["id"], []) if same_name(nodes, "link-type", l["type"], it["type"])]
        if not all(all(present((l.get("fields") or {}).get(f)) for f in it["fields"]) for l in links):
            outcome = "fail"
    return outcome


def check_any_of(nodes, inbound, node, val, calc=None):
    """A non-empty list of checks (each a non-empty map): passes if one passes. If none passes and one is not understood, the whole is not."""
    if not isinstance(val, list) or not val or not all(isinstance(it, dict) and it for it in val):
        return "bad"
    results = [run_check(nodes, inbound, node, it, calc) for it in val]
    return "pass" if "pass" in results else "bad" if "unsupported" in results else "fail"


def run_check(nodes, inbound, node, check, calc=None):
    """Returns pass, fail or unsupported. A check passes only if every part passes and every part is understood."""
    parts = []
    for key, val in check.items():
        if key == "require_fields":
            parts.append(check_require_fields(node, val))
        elif key == "field_is":
            parts.append(check_field_is(node, val, calc))
        elif key == "compare_fields":
            parts.append(check_compare_fields(node, val))
        elif key == "require_link":
            parts.append(check_require_link(nodes, inbound, node, val))
        elif key == "forbid_link":
            parts.append(check_forbid_link(nodes, inbound, node, val))
        elif key == "require_link_fields":
            parts.append(check_require_link_fields(nodes, inbound, node, val))
        elif key == "path":
            parts.append(check_path(nodes, inbound, node, val))
        elif key == "acyclic":
            parts.append(check_acyclic(nodes, node, val))
        elif key == "any_of":
            parts.append(check_any_of(nodes, inbound, node, val, calc))
        else:
            parts.append("bad")
    return "fail" if "fail" in parts else "unsupported" if "bad" in parts else "pass"


def ghosts_for(nodes, inbound, node, rule_id, check):
    """One ghost per outgoing require_link that has fewer links than its minimum (at least one)."""
    out = []
    if "require_link" not in check:
        return out
    for it in as_list(check["require_link"]):
        if not link_item_valid(it) or it.get("dir", "out") != "out":
            continue
        lo, _ = link_bounds(it)
        if lo >= 1 and count_links(nodes, inbound, node, it) < lo:
            ntype = it["node_type"] if "node_type" in it else it.get("to_type")
            nplane = it["node_plane"] if "node_plane" in it else it.get("to_plane")
            out.append({"node": node["id"], "rule": rule_id, "link": it["type"], "to_type": ntype,
                        "plane": nplane or node["plane"]})
    return out


def calculated(derived):
    """A node's derived values as plain values for rules: one value is text, several are a list. A name that could not be worked out is left out."""
    out = {}
    for nid, row in derived.items():
        vals = {name: (d["values"][0] if len(d["values"]) == 1 else list(d["values"])) for name, d in row.items() if d.get("values")}
        if vals:
            out[nid] = vals
    return out


def evaluate(nodes, derived=None):
    rules = sorted((n for n in nodes.values() if n["type"] == "rule"), key=lambda n: n["id"])
    inbound = {}
    for src in sorted(nodes):
        for l in nodes[src]["links"]:
            inbound.setdefault(l["to"], []).append((src, l))
    results, ghosts = [], []
    calc = calculated(derive(nodes) if derived is None else derived)
    for node in sorted(nodes.values(), key=lambda n: n["id"]):
        if node["type"] in RESERVED or scratch_of(nodes, node) is not None:
            continue
        for r in rules:
            f = r["fields"]
            if f.get("enabled") is False:   # switched off: no results, no ghosts
                continue
            _CTX[0] = def_ns(r)
            if not matches(nodes, node, f.get("applies_to") or {}):
                continue
            check = f.get("check")
            if not isinstance(check, dict) or not check:
                status = "advisory"
            else:
                status = run_check(nodes, inbound, node, check, calc.get(node["id"]))
            sev = f.get("severity")
            sev = sev if sev in ("error", "warn", "info") else "warn"
            results.append({"node": node["id"], "rule": r["id"], "severity": sev, "status": status})
            if status == "fail":
                ghosts += ghosts_for(nodes, inbound, node, r["id"], check)
    _CTX[0] = None
    key = lambda x: (x["node"], x["rule"])
    return sorted(results, key=key), sorted(ghosts, key=key)


def unmatched_rules(nodes):
    """Enabled rules that name specific nodes with `id` but match none of them: a typo looks exactly like success."""
    out = []
    for r in sorted((n for n in nodes.values() if n["type"] == "rule"), key=lambda n: n["id"]):
        f = r["fields"]
        where = f.get("applies_to")
        if f.get("enabled") is False or not isinstance(where, dict) or "id" not in where:
            continue
        _CTX[0] = def_ns(r)
        found = any(n["type"] not in RESERVED and scratch_of(nodes, n) is None and matches(nodes, n, where) for n in nodes.values())
        _CTX[0] = None
        if not found:
            out.append({"node": r["id"], "kind": "rule_matches_nothing", "ref": None})
    return out


# ---------- derived values ----------
MAX_DERIVED = 64   # the most values one derived name gives a node
MAX_PLACEHOLDERS = 32   # a template with more placeholders is unusable
MAX_HOPS = 5   # a placeholder that follows more links than this makes the template unusable
MAX_VALUE_CHARS = 1024   # a template that would build a longer value is unusable


def parse_template(t):
    """Split a template into ("lit", text) and ("ph", path) parts. `\\{`, `\\}` and `\\\\` are literal characters.
    None if a brace is unbalanced or a placeholder is empty."""
    parts, lit, i = [], "", 0
    while i < len(t):
        c = t[i]
        if c == "\\":
            if i + 1 < len(t) and t[i + 1] in "{}\\":
                lit += t[i + 1]
                i += 2
            else:
                lit += "\\"
                i += 1
        elif c == "{":
            j = i + 1
            while j < len(t) and t[j] not in "{}":
                j += 1
            if j >= len(t) or t[j] != "}" or j == i + 1:
                return None
            if lit:
                parts.append(("lit", lit))
                lit = ""
            if sum(1 for k, _ in parts if k == "ph") >= MAX_PLACEHOLDERS:
                return None
            if t[i + 1:j].count("/") > MAX_HOPS:
                return None
            parts.append(("ph", t[i + 1:j]))
            i = j + 1
        elif c == "}":
            return None
        else:
            lit += c
            i += 1
    if lit:
        parts.append(("lit", lit))
    return parts


def derived_text(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return json.dumps(v)
    if isinstance(v, str):
        return v
    return None


def derived_read(node, path):
    if path not in node["fields"]:
        if path == "id":
            return [node["id"]]
        if path == "title":
            return [node["title"]]
        if path == "@type":
            return [node["type"]] if node["type"] else []
    slots = resolve(node["fields"], path) or []
    return [t for t in (derived_text(s) for s in slots if s is not MISSING) if t is not None]


def derived_reach(nodes, inbound, types, node, hops):
    cur = {node["id"]: node}
    for hop in hops:
        nxt = {}
        for n in cur.values():
            if hop == "@type":
                found = candidates(nodes, "type", n["type"])
                t = found[0] if found else None
                if t is not None:
                    nxt[t["id"]] = t
            elif hop.startswith("^"):
                for src, l in inbound.get(n["id"], []):
                    if same_name(nodes, "link-type", l["type"], hop[1:]):
                        nxt[src] = nodes[src]
            else:
                for l in n["links"]:
                    if same_name(nodes, "link-type", l["type"], hop) and l["to"] in nodes:
                        nxt[l["to"]] = nodes[l["to"]]
        cur = nxt
    return [cur[k] for k in sorted(cur)]


def derived_placeholder(nodes, inbound, types, node, ph):
    segs = ph.split("/")
    if any(s == "" for s in segs):
        return []
    reached = derived_reach(nodes, inbound, types, node, segs[:-1])
    return [v for n in reached for v in derived_read(n, segs[-1])]


def derived_evaluate(nodes, inbound, types, node, parts):
    if not parts:
        return {"missing": ["template"]}
    results, missing = [""], []
    for kind, val in parts:
        if kind == "lit":
            if any(len(r) + len(val) > MAX_VALUE_CHARS for r in results):
                return {"missing": ["template"]}
            results = [r + val for r in results]
            continue
        vals = derived_placeholder(nodes, inbound, types, node, val)
        if not vals:
            if val not in missing:
                missing.append(val)
            continue
        nxt = []
        for r in results:
            for v in vals:
                if len(nxt) >= MAX_DERIVED:
                    break
                if len(r) + len(v) > MAX_VALUE_CHARS:
                    return {"missing": ["template"]}
                nxt.append(r + v)
            if len(nxt) >= MAX_DERIVED:
                break
        results = nxt
    if missing:
        return {"missing": missing}
    seen = []
    for r in results:
        if r not in seen:
            seen.append(r)
    return {"values": seen}


def derived_override(node, name, computed):
    """What the node says its name is (`overrides: {name: "value"}` or a list of values) replaces the template; `computed` keeps what
    the template gave (empty when it could not be resolved). Anything but non-empty strings is ignored."""
    given = node["fields"].get("overrides")
    given = given.get(name) if isinstance(given, dict) else None
    given = given if isinstance(given, list) else [given]
    values = []
    for v in given:
        if isinstance(v, str) and v != "" and v not in values and len(values) < 64:
            values.append(v)
    if not values:
        return computed
    return {"values": values, "override": True, "computed": computed.get("values", [])}


def derive(nodes):
    """Every node's derived values: {node id: {name: {"values": [...]} or {"missing": [...]}}}. Reserved nodes and nodes in a
    scratch have none. For each name the most specific applicable definition wins (id 3, type 2, plane 1, other 0); ties go to the
    lower definition id."""
    defs = []
    for d in sorted((n for n in nodes.values() if n["type"] == "derived"), key=lambda n: n["id"]):
        f = d["fields"]
        if f.get("enabled") is False or not is_str(f.get("name")) or not isinstance(f.get("template"), str):
            continue
        where = f.get("applies_to") if isinstance(f.get("applies_to"), dict) else {}
        spec = 3 if "id" in where else 2 if "type" in where else 1 if "plane" in where else 0
        # `name` is the definition's own; `field-name`, when set, is what the value is called on a node
        defs.append((f["field-name"] if is_str(f.get("field-name")) else f["name"], parse_template(f["template"]), where, spec, def_ns(d)))
    out = {}
    if not defs:
        return out
    inbound = {}
    for src in sorted(nodes):
        for l in nodes[src]["links"]:
            inbound.setdefault(l["to"], []).append((src, l))
    types = None   # a node's type definition is found through `candidates`
    for node in sorted(nodes.values(), key=lambda n: n["id"]):
        if node["type"] in RESERVED or scratch_of(nodes, node) is not None:
            continue
        best = {}
        for name, parts, where, spec, ns in defs:
            _CTX[0] = ns
            if not matches(nodes, node, where):
                continue
            if name not in best or spec > best[name][1]:
                best[name] = (parts, spec, ns)
        if best:
            values = {}
            for name in sorted(best):
                _CTX[0] = best[name][2]
                values[name] = derived_override(node, name, derived_evaluate(nodes, inbound, types, node, best[name][0]))
            out[node["id"]] = values
    _CTX[0] = None
    return out


# ---------- output ----------
def check(root):
    nodes, load_errors = load(root)
    errors, warnings = structure(nodes)
    warnings = sorted(warnings + unmatched_rules(nodes), key=lambda f: (f["node"], f["kind"], f["ref"] or ""))
    derived = derive(nodes)
    results, ghosts = evaluate(nodes, derived)
    return {
        "load_errors": sorted(load_errors, key=lambda e: (e["file"], e["kind"])),
        "nodes": [({**nodes[i], "derived": derived[i]} if i in derived else nodes[i]) for i in sorted(nodes)],
        "structure": {"errors": errors, "warnings": warnings},
        "results": results,
        "ghosts": ghosts,
    }


def exit_code(out):
    bad = out["load_errors"] or out["structure"]["errors"] or any(
        r["status"] == "fail" and r["severity"] == "error" for r in out["results"])
    return 1 if bad else 0


def print_human(out):
    for e in out["load_errors"]:
        print(f"ERROR  {e['file']}: {e['kind']}")
    for e in out["structure"]["errors"]:
        print(f"ERROR  {e['node']}: {e['kind']} {e['ref'] or ''}".rstrip())
    for w in out["structure"]["warnings"]:
        print(f"warn   {w['node']}: {w['kind']} {w['ref'] or ''}".rstrip())
    for r in out["results"]:
        if r["status"] == "fail":
            print(f"{r['severity']:<6} {r['node']}: fails {r['rule']}")
    print(f"{len(out['nodes'])} nodes, {len(out['load_errors'])} load errors, "
          f"{len(out['structure']['errors'])} errors, {len(out['structure']['warnings'])} warnings")


if __name__ == "__main__":
    a = [x for x in sys.argv[1:] if x != "--json"]
    if len(a) == 2 and a[0] == "check":
        out = check(a[1])
        if "--json" in sys.argv:
            json.dump(out, sys.stdout, indent=1, ensure_ascii=False, sort_keys=True)
            print()
        else:
            print_human(out)
        sys.exit(exit_code(out))
    print(__doc__)
    sys.exit(2)
