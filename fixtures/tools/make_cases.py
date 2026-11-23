#!/usr/bin/env python3
"""Writes fixtures/cases/<name>/ (input .md files) from the table below.
expected.json is produced separately by `bless.py` and then REVIEWED BY HAND before commit."""
import shutil, sys
from pathlib import Path

CASES = Path(__file__).resolve().parents[1] / "cases"

def n(id, type=None, parent=None, plane=None, fields=None, links=None, title=None):
    lines = [f"id: {id}"]
    if type: lines.append(f"type: {type}")
    if title: lines.append(f"title: {title}")
    if parent: lines.append(f"parent: {parent}")
    if plane: lines.append(f"plane: {plane}")
    if fields:
        lines.append("fields:")
        lines += [f"  {k}: {v}" for k, v in fields.items()]
    if links:
        lines.append("links:")
        for l in links:
            if isinstance(l, dict):
                lines.append(f"  - type: {l['type']}")
                lines.append(f"    to: {l['to']}")
                if l.get("key"): lines.append(f"    key: {l['key']}")
                if l.get("fields"): lines.append(f"    fields: {l['fields']}")
                continue
            lines.append(f"  - type: {l[0]}")
            lines.append(f"    to: {l[1]}")
            if len(l) > 2: lines.append(f"    key: {l[2]}")
    return "---\n" + "\n".join(lines) + "\n---\n"

def rule(id, applies=None, check=None, severity=None, extra="", enabled=None):
    f = {}
    if severity: f["severity"] = severity
    out = ["id: " + id, "type: rule", "fields:", "  statement: s"]
    if severity: out.append(f"  severity: {severity}")
    if enabled is not None: out.append(f"  enabled: {enabled}")
    if applies: out.append("  applies_to: " + applies)
    if check: out.append("  check: " + check)
    return "---\n" + "\n".join(out) + "\n---\n"

PL = lambda id, order: n(id, "plane", fields={"order": order})

cases = {
 "valid-minimal": {"a.md": n("a", "thing")},
 "missing-parent": {"a.md": n("a", "thing", parent="ghost")},
 "parent-cycle": {"a.md": n("a", "t", parent="b"), "b.md": n("b", "t", parent="a"), "c.md": n("c", "t", parent="a")},
 "parent-chain-missing": {"a.md": n("a", "t", parent="b"), "b.md": n("b", "t", parent="gone"),
     "r.md": rule("r", applies="{under: gone}")},
 "missing-link-target": {"a.md": n("a", "t", links=[("uses", "nowhere")])},
 "duplicate-link": {"a.md": n("a", "t", links=[("uses", "b"), ("uses", "b"), ("uses", "b", "k"), ("uses", "b", "k")]), "b.md": n("b", "t")},
 "undeclared-plane": {"a.md": n("a", "t", plane="nope"), "p.md": PL("p", 1), "b.md": n("b", "t", plane="p")},
 "missing-type": {"a.md": n("a")},
 "rule-require-fields": {
     "a.md": n("a", "svc", fields={"owner": "x"}), "b.md": n("b", "svc"), "c.md": n("c", "svc", fields={"owner": "''"}),
     "r.md": rule("r", applies="{type: svc}", check="{require_fields: [owner]}", severity="error")},
 "rule-require-link": {
     "p1.md": PL("p1", 1), "p2.md": PL("p2", 2),
     "a.md": n("a", "svc", plane="p1", links=[("runs_on", "e")]), "b.md": n("b", "svc", plane="p1"),
     "c.md": n("c", "svc", plane="p1", links=[("runs_on", "other")]),
     "d.md": n("d", "svc", plane="p1", links=[("runs_on", "nowhere")]),
     "e.md": n("e", "env", plane="p2"), "other.md": n("other", "svc", plane="p2"),
     "r.md": rule("r", applies="{type: svc}", check="{require_link: {type: runs_on, to_type: env, to_plane: p2}}")},
 "rule-applies-to": {
     "g.md": n("g", "group"), "a.md": n("a", "svc", parent="g", fields={"tier": 1}),
     "b.md": n("b", "svc", parent="a", fields={"tier": "true"}), "c.md": n("c", "svc", fields={"tier": "true"}),
     "d.md": n("d", "svc", fields={"tier": "null"}), "e.md": n("e", "svc"),
     "r1.md": rule("r1", applies="{under: g}", check="{require_fields: [zz]}"),
     "r2.md": rule("r2", applies="{tier: true}", check="{require_fields: [zz]}"),
     "r3.md": rule("r3", applies="{tier: null}", check="{require_fields: [zz]}"),
     "r4.md": rule("r4", applies="{tier: 1}", check="{require_fields: [zz]}")},
 "rule-advisory": {
     "a.md": n("a", "svc"), "r1.md": rule("r1", applies="{type: svc}"), "r2.md": rule("r2", applies="{type: svc}", check="{}"),
     "r3.md": rule("r3", applies="{type: svc}", check="{mystery: 1}", severity="info")},
 "reserved-exempt": {
     "p.md": PL("p", 1), "g.md": n("g", "group"), "a.md": n("a", "svc"),
     "r.md": rule("r", check="{require_fields: [owner]}", severity="error")},
 "rule-disabled": {
     "p.md": PL("p", 1), "a.md": n("a", "svc", plane="p"),
     "off.md": rule("off", applies="{type: svc}", check="{require_fields: [owner]}", severity="error", enabled="false"),
     "on.md": rule("on", applies="{type: svc}", check="{require_fields: [owner]}", enabled="true"),
     "str.md": rule("str", applies="{type: svc}", check="{require_link: {type: needs}}", enabled="'false'"),
     "off-no-check.md": rule("off-no-check", applies="{type: svc}", enabled="false")},
 "reserved-definition-kinds": {
     "p.md": PL("p", 1), "a.md": n("a", "svc", plane="p"),
     "type-svc.md": n("type-svc", "type", fields={"name": "svc"}),
     "link-type-needs.md": n("link-type-needs", "link-type", fields={"name": "needs"}),
     "pack-x.md": n("pack-x", "pack", fields={"name": "x", "version": "'1.0.0'"}),
     "r.md": rule("r", check="{require_fields: [owner]}", severity="error")},
 "check-precedence": {
     "p.md": PL("p", 1), "a.md": n("a", "svc", plane="p"),
     "r.md": rule("r", applies="{type: svc}", check="{require_fields: [owner], require_link: {type: x}}")},
 "ghost-plane-fallback": {
     "p.md": PL("p", 1), "a.md": n("a", "svc", plane="p"), "b.md": n("b", "svc"),
     "r.md": rule("r", applies="{type: svc}", check="{require_link: {type: needs}}")},
 "format-body": {
     "a.md": "---\r\nid: a\r\ntype: doc\r\nunknown_key: ignored\r\n---\r\n\r\n# Heading\r\n\r\nText with --- inside.\r\n---\r\nmore\r\n\r\n",
     "b.md": "---\nid: b\ntype: doc\n---\n"},
 "load-invalid-frontmatter": {"a.md": "no frontmatter here\n", "b.md": "---\nid: [unclosed\n---\n", "c.md": "---\n- just\n- a list\n---\n", "ok.md": n("ok", "t")},
 "load-missing-id": {"a.md": "---\ntype: t\n---\n", "ok.md": n("ok", "t")},
 "load-id-mismatch": {"a.md": n("b", "t"), "ok.md": n("ok", "t")},
 "load-duplicate-id": {"x.md": n("x", "t", title="second"), "sub/x.md": n("x", "t", title="first")},
 "load-subdirs": {"sub/deep/a.md": n("a", "t"), "b.md": n("b", "t", parent="a")},
 "values-dates-are-strings": {"a.md": "---\nid: a\ntype: t\nfields:\n  due: 2026-10-19\n  at: 2026-10-19T16:00:00Z\n---\n"},
 "rule-under-strict": {"a.md": n("a", "svc"), "b.md": n("b", "svc", parent="a"), "c.md": n("c", "svc", parent="b"),
     "r.md": rule("r", applies="{under: a}", check="{require_fields: [zz]}")},
 "rule-bad-severity": {"a.md": n("a", "svc"),
     "r1.md": rule("r1", applies="{type: svc}", check="{require_fields: [zz]}", severity="loud"),
     "r2.md": rule("r2", applies="{type: svc}", check="{require_fields: [zz]}", severity="error")},
 "values-yaml12": {"a.md": "---\nid: a\ntype: t\nfields:\n  switch: on\n  answer: no\n  mode: Yes\n  perm: 0755\n  hex: 0x1F\n  sci: 1e3\n  big: 9223372036854775808\n  neg: -12\n  t: True\n  nul: ~\nlinks:\n  - type: x\n    to: a\n    fields: {on: go, off: stop}\n---\n"},
 "values-numbers": {
     "a.md": n("a", "t", fields={"v": 1}), "b.md": n("b", "t", fields={"v": 1.0}), "c.md": n("c", "t", fields={"v": 1.5}),
     "d.md": n("d", "t", fields={"v": "'1'"}), "e.md": n("e", "t", fields={"v": "[1, true]"}),
     "r1.md": rule("r1", applies="{v: 1}", check="{require_fields: [zz]}"),
     "r2.md": rule("r2", applies="{v: [1, true]}", check="{require_fields: [zz]}"),
     "r3.md": rule("r3", applies="{v: 1.5}", check="{require_fields: [zz]}")},
 "load-not-json": {"a.md": "---\nid: a\ntype: t\nfields:\n  x: .nan\n---\n", "c.md": "---\nid: c\ntype: t\nfields:\n  1: one\n---\n", "ok.md": n("ok", "t")},
 "load-bad-field-types": {"a.md": "---\nid: a\ntype: t\nparent: 5\n---\n", "b.md": "---\nid: b\ntype: t\nlinks:\n  - type: x\n---\n", "ok.md": n("ok", "t")},
}

L = lambda type, to, fields=None, key=None: {"type": type, "to": to, "fields": fields, "key": key}

cases.update({
 "check-and-semantics": {
     "a.md": n("a", "svc", fields={"owner": "x", "tier": 1}), "b.md": n("b", "svc", fields={"owner": "x", "tier": 3}),
     "c.md": n("c", "svc", fields={"tier": 1}),
     "r.md": rule("r", applies="{type: svc}", check="{require_fields: [owner], field_is: {tier: {in: [1, 2]}}}")},
 "check-unsupported": {
     "a.md": n("a", "svc", fields={"owner": "x"}), "b.md": n("b", "svc"),
     "r1.md": rule("r1", applies="{type: svc}", check="{mystery: 1}"),
     "r2.md": rule("r2", applies="{type: svc}", check="{require_fields: [owner], mystery: 1}"),
     "r3.md": rule("r3", applies="{type: svc}", check="{require_fields: owner}"),
     "r4.md": rule("r4", applies="{type: svc}", check="{field_is: {x: {bogus: 1}}}"),
     "r5.md": rule("r5", applies="{type: svc}", check="{compare_fields: {left: a, op: '~', right: b}}")},
 "check-field-is": {
     "a.md": n("a", "t", fields={"v": 1}), "b.md": n("b", "t", fields={"v": 3}), "c.md": n("c", "t", fields={"v": "'2'"}),
     "d.md": n("d", "t"), "e.md": n("e", "t", fields={"v": 2.5}), "f.md": n("f", "t", fields={"v": "true"}),
     "r1.md": rule("r1", applies="{type: t}", check="{field_is: {v: {in: [1, 2]}}}"),
     "r2.md": rule("r2", applies="{type: t}", check="{field_is: {v: {min: 2, max: 3}}}"),
     "r3.md": rule("r3", applies="{type: t}", check="{field_is: {v: {type: integer}}}"),
     "r4.md": rule("r4", applies="{type: t}", check="{field_is: {v: {type: number, optional: true}}}"),
     "r5.md": rule("r5", applies="{type: t}", check="{field_is: {v: {type: boolean}}}")},
 "check-field-paths": {
     "a.md": n("a", "api", fields={"endpoint": "{path: /users, params: [id]}", "methods": "[{name: list, verb: GET}, {name: add, verb: POST}]", "tags": "[x, y]"}),
     "b.md": n("b", "api", fields={"endpoint": "{path: /x}", "methods": "[{name: list, verb: GET}, {name: list, verb: FETCH}]", "tags": "[x, x]"}),
     "c.md": n("c", "api", fields={"endpoint": "{}", "methods": "[]", "tags": "[]"}),
     "d.md": n("d", "api", fields={"endpoint": "'text'", "methods": "[{name: a}, 3]", "tags": "x"}),
     "e.md": n("e", "api"),
     "f.md": n("f", "api", fields={"a.b": 1}),
     "r1.md": rule("r1", applies="{type: api}", check="{require_fields: [endpoint.path]}"),
     "r2.md": rule("r2", applies="{type: api}", check="{require_fields: ['methods[].verb']}"),
     "r3.md": rule("r3", applies="{type: api}", check="{field_is: {'methods[].verb': {in: [GET, POST, PUT, DELETE]}}}"),
     "r4.md": rule("r4", applies="{type: api}", check="{field_is: {methods: {min_items: 1}}}"),
     "r5.md": rule("r5", applies="{type: api}", check="{field_is: {methods: {unique: name}, tags: {unique: true, optional: true}}}"),
     "r6.md": rule("r6", applies="{type: api}", check="{field_is: {methods: {type: list, max_items: 1}}}"),
     "r7.md": rule("r7", applies="{type: api}", check="{compare_fields: {left: 'methods[].verb', op: '!=', value: HEAD}}"),
     "r8.md": rule("r8", applies="{type: api}", check="{require_fields: [a.b]}"),
     "r9.md": rule("r9", applies="{type: api}", check="{field_is: {'tags[]': {type: string}}}"),
     "r10.md": rule("r10", applies="{type: api}", check="{require_fields: ['methods[]..x']}"),
     "r11.md": rule("r11", applies="{type: api}", check="{field_is: {methods: {unique: 3}}}"),
     "r12.md": rule("r12", applies="{type: api}", check="{compare_fields: {left: endpoint.path, op: '==', right: 'methods[].verb'}}"),
     "r13.md": rule("r13", applies="{type: api, 'methods[].verb': POST}", check="{require_fields: [endpoint]}"),
     "r14.md": rule("r14", applies="{type: api, endpoint.path: /x}", check="{require_fields: [tags]}"),
     "r15.md": rule("r15", applies="{'methods[]..verb': GET}", check="{require_fields: [tags]}")},
 "applies-to-any-of": {
     "p1.md": PL("p1", 1), "p2.md": PL("p2", 2),
     "g.md": n("g", "group"),
     "a.md": n("a", "svc", plane="p1", parent="g", fields={"tier": "gold", "methods": "[{verb: GET}, {verb: PUT}]", "tags": "[x, y]"}),
     "b.md": n("b", "queue", plane="p2", fields={"tier": "silver", "methods": "[{verb: GET}]", "tags": "[z]"}),
     "c.md": n("c", "cache", plane="p2", parent="a", fields={"tier": "bronze", "methods": "[{verb: DELETE}]", "tags": "[]"}),
     "d.md": n("d", "other", plane="p1", fields={"tier": "gold"}),
     "r1.md": rule("r1", applies="{type: [svc, queue]}", check="{require_fields: [owner]}"),
     "r2.md": rule("r2", applies="{type: [svc, queue], plane: [p2, p9]}", check="{require_fields: [owner]}"),
     "r3.md": rule("r3", applies="{tier: [gold, silver]}", check="{require_fields: [owner]}"),
     "r4.md": rule("r4", applies="{'methods[].verb': [PUT, DELETE]}", check="{require_fields: [owner]}"),
     "r5.md": rule("r5", applies="{'tags[]': [x, z]}", check="{require_fields: [owner]}"),
     "r6.md": rule("r6", applies="{under: [g, a]}", check="{require_fields: [owner]}"),
     "r7.md": rule("r7", applies="{id: [a, d, nope]}", check="{require_fields: [owner]}"),
     "r8.md": rule("r8", applies="{type: []}", check="{require_fields: [owner]}"),
     "r9.md": rule("r9", applies="{type: svc}", check="{require_fields: [owner]}")},
 "check-compare-fields": {
     "a.md": n("a", "t", fields={"seen": 0.97, "target": 0.95}), "b.md": n("b", "t", fields={"seen": 0.9, "target": 0.95}),
     "c.md": n("c", "t", fields={"seen": "'x'", "target": 0.95}), "d.md": n("d", "t", fields={"seen": 0.99}),
     "e.md": n("e", "t", fields={"seen": 1, "target": 1.0}),
     "f.md": n("f", "t", fields={"seen": "2026-10-19T10:00:00Z", "target": "2026-10-19T09:00:00Z"}),
     "r1.md": rule("r1", applies="{type: t}", check="{compare_fields: {left: seen, op: '>=', right: target}}"),
     "r2.md": rule("r2", applies="{type: t}", check="{compare_fields: [{left: seen, op: '==', right: target}, {left: seen, op: '!=', value: 0}]}"),
     "r3.md": rule("r3", applies="{type: t}", check="{compare_fields: {left: seen, op: '<', value: 1}}")},
 "check-link-cardinality": {
     "a.md": n("a", "svc", links=[("uses", "x"), ("uses", "y")]), "b.md": n("b", "svc", links=[("uses", "x")]), "c.md": n("c", "svc"),
     "x.md": n("x", "dep"), "y.md": n("y", "dep"),
     "r1.md": rule("r1", applies="{type: svc}", check="{require_link: {type: uses, min: 2}}"),
     "r2.md": rule("r2", applies="{type: svc}", check="{require_link: {type: uses, max: 1}}"),
     "r3.md": rule("r3", applies="{type: svc}", check="{require_link: {type: uses, max: 0}}"),
     "r4.md": rule("r4", applies="{type: svc}", check="{require_link: {type: uses, node_type: dep, min: 1, max: 1}}")},
 "check-link-inbound": {
     "a.md": n("a", "svc", links=[("calls", "db")]), "b.md": n("b", "job", links=[("calls", "db")]), "db.md": n("db", "store"), "lonely.md": n("lonely", "store"),
     "r1.md": rule("r1", applies="{type: store}", check="{require_link: {type: calls, dir: in}}"),
     "r2.md": rule("r2", applies="{type: store}", check="{require_link: {type: calls, dir: in, node_type: svc, max: 1}}"),
     "r3.md": rule("r3", applies="{type: store}", check="{require_link: {type: calls, dir: in, node_type: job, max: 0}}")},
 "check-link-scope": {
     "d1.md": n("d1", "domain"), "d2.md": n("d2", "domain"),
     "c1.md": n("c1", "component", parent="d1", links=[("accesses", "s1"), ("accesses", "s2")]),
     "c2.md": n("c2", "component", parent="d2", links=[("accesses", "s2")]),
     "c3.md": n("c3", "component", links=[("accesses", "s1")]),
     "s1.md": n("s1", "store", parent="d1"), "s2.md": n("s2", "store", parent="d2"),
     "r1.md": rule("r1", applies="{type: component}", check="{require_link: {type: accesses, ancestor_type: domain, scope: other, max: 0}}"),
     "r2.md": rule("r2", applies="{type: component}", check="{require_link: {type: accesses, ancestor_type: domain, scope: same}}")},
 "check-link-fields": {
     "a.md": n("a", "state", links=[L("goes", "b", "{on: go}"), L("goes", "c", "{on: stop}")]),
     "b.md": n("b", "state", links=[L("goes", "a", "{on: ''}"), L("goes", "c")]), "c.md": n("c", "state", links=[L("goes", "a", "{on: go}")]),
     "d.md": n("d", "state", links=[L("other", "a")]),
     "r1.md": rule("r1", applies="{type: state}", check="{require_link_fields: {type: goes, fields: [on]}}"),
     "r2.md": rule("r2", applies="{type: state}", check="{require_link: {type: goes, with_fields: [on], min: 2}}")},
 "ghost-multiple": {
     "p.md": PL("p", 1), "q.md": PL("q", 2), "a.md": n("a", "svc", plane="p", links=[("uses", "x")]), "x.md": n("x", "dep", plane="q"),
     "r.md": rule("r", applies="{type: svc}", check="{require_link: [{type: owns, node_type: team, node_plane: q}, {type: uses, min: 2}, {type: ok}]}")},
})

# ---- stress scenarios: models of real structures, written the way we want them to read ----
cases.update({
 # A task lifecycle as a state machine: states are nodes, transitions are links carrying the event.
 "stress-task-lifecycle": {
     "p.md": PL("p", 1),
     "requested.md": n("requested", "state", plane="p", fields={"initial": "true", "terminal": "false"}, links=[L("transitions-to", "accepted", "{on: accept}"), L("transitions-to", "declined", "{on: decline}")]),
     "accepted.md": n("accepted", "state", plane="p", fields={"initial": "false", "terminal": "false"}, links=[L("transitions-to", "delivered", "{on: deliver}"), L("transitions-to", "cancelled")]),
     "delivered.md": n("delivered", "state", plane="p", fields={"initial": "false", "terminal": "true"}),
     "declined.md": n("declined", "state", plane="p", fields={"initial": "false", "terminal": "true"}, links=[L("transitions-to", "requested", "{on: retry}")]),
     "cancelled.md": n("cancelled", "state", plane="p", fields={"initial": "false", "terminal": "true"}),
     "stuck.md": n("stuck", "state", plane="p", fields={"initial": "false", "terminal": "false"}),
     "orphan.md": n("orphan", "state", plane="p", fields={"initial": "false", "terminal": "true"}),
     "r-exit.md": rule("r-exit", applies="{type: state, terminal: false}", check="{require_link: {type: transitions-to}}", severity="error"),
     "r-final.md": rule("r-final", applies="{type: state, terminal: true}", check="{require_link: {type: transitions-to, max: 0}}", severity="error"),
     "r-entry.md": rule("r-entry", applies="{type: state, initial: false}", check="{require_link: {type: transitions-to, dir: in}}"),
     "r-event.md": rule("r-event", applies="{type: state}", check="{require_link_fields: {type: transitions-to, fields: [on]}}", severity="error")},
 # Observed, time-varying values live in fields that an agent or pipeline rewrites; rules compare them with targets.
 "stress-observed-data": {
     "p.md": PL("p", 1),
     "checkout-latency.md": n("checkout-latency", "metric", plane="p", fields={"unit": "ms", "observed": 180, "limit": 250, "status": "ok", "observed_at": "2026-10-19T09:30:00Z"}),
     "search-latency.md": n("search-latency", "metric", plane="p", fields={"unit": "ms", "observed": 410, "limit": 300, "status": "ok", "observed_at": "2026-10-19T09:30:00Z"}),
     "uptime.md": n("uptime", "metric", plane="p", fields={"unit": "ratio", "observed": 0.9991, "target": 0.999, "status": "ok", "observed_at": "2026-10-19T09:31:00Z"}),
     "error-rate.md": n("error-rate", "metric", plane="p", fields={"unit": "ratio", "observed": 0.02, "limit": 0.01, "status": "degraded"}),
     "queue-depth.md": n("queue-depth", "metric", plane="p", fields={"unit": "items", "observed": "n/a", "limit": 100, "status": "weird", "observed_at": "2026-10-19T09:30:00Z"}),
     "r-limit.md": rule("r-limit", applies="{type: metric, unit: ms}", check="{compare_fields: {left: observed, op: '<=', right: limit}}", severity="error"),
     "r-target.md": rule("r-target", applies="{type: metric, unit: ratio}", check="{compare_fields: {left: observed, op: '>=', right: target}}"),
     "r-status.md": rule("r-status", applies="{type: metric}", check="{field_is: {status: {in: [ok, degraded, down]}, observed_at: {type: string}}}"),
     "r-number.md": rule("r-number", applies="{type: metric}", check="{field_is: {observed: {type: number}}}")},
 # Rules that span types: traceability, deployment, and bounded contexts.
 "stress-cross-type": {
     "intent.md": PL("intent", 1), "build.md": PL("build", 2), "run.md": PL("run", 3),
     "billing.md": n("billing", "domain"), "catalogue.md": n("catalogue", "domain"),
     "req-pay.md": n("req-pay", "requirement", plane="intent", links=[L("implemented-by", "pay-api")]),
     "req-refund.md": n("req-refund", "requirement", plane="intent"),
     "req-search.md": n("req-search", "requirement", plane="intent", links=[L("implemented-by", "search-api")]),
     "pay-api.md": n("pay-api", "component", plane="build", parent="billing", links=[L("runs-in", "prod"), L("accesses", "ledger"), L("accesses", "products")]),
     "search-api.md": n("search-api", "component", plane="build", parent="catalogue", links=[L("accesses", "products")]),
     "ledger.md": n("ledger", "datastore", plane="build", parent="billing"),
     "products.md": n("products", "datastore", plane="build", parent="catalogue"),
     "prod.md": n("prod", "environment", plane="run"),
     "r-built.md": rule("r-built", applies="{type: requirement}", check="{require_link: {type: implemented-by, node_type: component, node_plane: build}}"),
     "r-runs.md": rule("r-runs", applies="{type: component}", check="{require_link: {type: runs-in, node_type: environment, node_plane: run}}"),
     "r-context.md": rule("r-context", applies="{type: component}", check="{require_link: {type: accesses, ancestor_type: domain, scope: other, max: 0}}", severity="error"),
     "r-traced.md": rule("r-traced", applies="{type: component}", check="{require_link: {type: implemented-by, dir: in, node_type: requirement}}")},
})

# ---- rules v0.2: id filters, far-end fields, forbid_link, path, acyclic; notes beside a model ----
cases.update({
 "rule-id-filter": {
     "a.md": n("a", "svc"), "b.md": n("b", "svc"), "c.md": n("c", "svc"),
     "r1.md": rule("r1", applies="{id: a}", check="{require_fields: [owner]}"),
     "r2.md": rule("r2", applies="{id: [b, c, zzz, 7]}", check="{require_fields: [owner]}"),
     "r3.md": rule("r3", applies="{id: zzz}", check="{require_fields: [owner]}"),
     "r4.md": rule("r4", applies="{id: 5}", check="{require_fields: [owner]}"),
     "r5.md": rule("r5", applies="{id: a, type: other}", check="{require_fields: [owner]}"),
     "r6.md": rule("r6", applies="{id: zzz}", check="{require_fields: [owner]}", enabled="false"),
     "r7.md": rule("r7", applies="{id: r1}", check="{require_fields: [owner]}")},
 "check-node-fields": {
     "ui.md": n("ui", "comp", fields={"layer": "app"}, links=[L("depends-on", "svc"), L("depends-on", "util")]),
     "svc.md": n("svc", "comp", fields={"layer": "domain"}, links=[L("depends-on", "util"), L("depends-on", "db")]),
     "util.md": n("util", "comp", fields={"layer": "pure"}),
     "db.md": n("db", "comp", fields={"layer": "infra"}),
     "bare.md": n("bare", "comp", links=[L("depends-on", "util")]),
     "r-pure-require.md": rule("r-pure-require", applies="{type: comp}", check="{require_link: {type: depends-on, node_fields: {layer: pure}}}"),
     "r-pure-only.md": rule("r-pure-only", applies="{type: comp, layer: domain}", check="{forbid_link: {type: depends-on, node_fields: {layer: infra}}}"),
     "r-forbid-in.md": rule("r-forbid-in", applies="{type: comp}", check="{forbid_link: [{type: depends-on, dir: in, node_type: comp, node_fields: {layer: app}}]}"),
     "r-forbid-bounds.md": rule("r-forbid-bounds", applies="{type: comp}", check="{forbid_link: {type: depends-on, max: 1}}"),
     "r-bad-fields.md": rule("r-bad-fields", applies="{type: comp}", check="{require_link: {type: depends-on, node_fields: [layer]}}")},
 "check-same-fields": {
     "a-int.md": n("a-int", "col", fields={"data_type": "integer"}, links=[L("references", "b-int")]),
     "b-int.md": n("b-int", "col", fields={"data_type": "integer"}),
     "c-text.md": n("c-text", "col", fields={"data_type": "text"}, links=[L("references", "b-int")]),
     "d-bare.md": n("d-bare", "col", links=[L("references", "b-int")]),
     "r-same.md": rule("r-same", applies="{type: col}", check="{require_link: {type: references, same_fields: [data_type], min: 0, max: 0}}"),
     "r-need.md": rule("r-need", applies="{id: [a-int, c-text]}", check="{require_link: {type: references, same_fields: [data_type]}}"),
     "r-bad.md": rule("r-bad", applies="{id: a-int}", check="{require_link: {type: references, same_fields: data_type}}")},
 "check-path": {
     "p1.md": PL("p1", 1),
     "req-a.md": n("req-a", "req", links=[L("built-by", "comp-a"), L("built-by", "comp-b")]),
     "req-b.md": n("req-b", "req", links=[L("built-by", "comp-b")]),
     "req-c.md": n("req-c", "req"),
     "req-d.md": n("req-d", "req", links=[L("built-by", "ghost")]),
     "comp-a.md": n("comp-a", "comp", fields={"status": "built"}, links=[L("runs-in", "prod")]),
     "comp-b.md": n("comp-b", "comp", fields={"status": "planned"}),
     "prod.md": n("prod", "env", plane="p1"),
     "r-chain.md": rule("r-chain", applies="{type: req}", check="{path: [{type: built-by, node_type: comp}, {type: runs-in, node_type: env, node_plane: p1}]}"),
     "r-built.md": rule("r-built", applies="{type: req}", check="{path: [{type: built-by, node_fields: {status: built}}]}"),
     "r-back.md": rule("r-back", applies="{type: env}", check="{path: [{type: runs-in, dir: in}, {type: built-by, dir: in, node_type: req}]}"),
     "r-empty.md": rule("r-empty", applies="{type: env}", check="{path: []}"),
     "r-badstep.md": rule("r-badstep", applies="{type: env}", check="{path: [{type: runs-in, min: 1}]}"),
     "r-notlist.md": rule("r-notlist", applies="{type: env}", check="{path: {type: runs-in}}")},
 "check-acyclic": {
     "a.md": n("a", "t", links=[L("dep", "b"), L("other", "a")]),
     "b.md": n("b", "t", links=[L("dep", "c")]),
     "c.md": n("c", "t", links=[L("dep", "a")]),
     "d.md": n("d", "t", links=[L("dep", "a")]),
     "e.md": n("e", "t", links=[L("dep", "e")]),
     "f.md": n("f", "t", links=[L("dep", "gone")]),
     "g.md": n("g", "t"),
     "r-dep.md": rule("r-dep", applies="{type: t}", check="{acyclic: dep}", severity="error"),
     "r-list.md": rule("r-list", applies="{type: t}", check="{acyclic: [dep, other]}"),
     "r-bad.md": rule("r-bad", applies="{type: t}", check="{acyclic: [dep, 3]}"),
     "r-map.md": rule("r-map", applies="{type: t}", check="{acyclic: {type: dep}}")},
 "check-any-of": {
     "n1.md": n("n1", "t", links=[L("a", "t3")]),
     "n2.md": n("n2", "t", links=[L("b", "t3")]),
     "n3.md": n("n3", "t"),
     "n4.md": n("n4", "t", fields={"x": 1}),
     "t3.md": n("t3", "u"),
     "r-either.md": rule("r-either", applies="{type: t}", check="{any_of: [{require_link: {type: a}}, {require_link: {type: b}}]}", severity="error"),
     "r-fields-or-link.md": rule("r-fields-or-link", applies="{type: t}", check="{any_of: [{require_fields: [x]}, {require_link: {type: a}}]}"),
     "r-and-inside.md": rule("r-and-inside", applies="{type: t}", check="{any_of: [{require_fields: [x], require_link: {type: a}}, {require_link: {type: b}}]}"),
     "r-nested.md": rule("r-nested", applies="{type: t}", check="{any_of: [{any_of: [{require_link: {type: a}}, {require_link: {type: b}}]}, {require_fields: [x]}]}"),
     "r-with-bad-part.md": rule("r-with-bad-part", applies="{type: t}", check="{any_of: [{require_link: {type: a}}, {nonsense: 1}]}"),
     "r-empty.md": rule("r-empty", applies="{type: t}", check="{any_of: []}"),
     "r-not-list.md": rule("r-not-list", applies="{type: t}", check="{any_of: {require_link: {type: a}}}"),
     "r-empty-item.md": rule("r-empty-item", applies="{type: t}", check="{any_of: [{}, {require_link: {type: a}}]}"),
     "r-with-sibling.md": rule("r-with-sibling", applies="{type: t}", check="{any_of: [{require_link: {type: b}}], require_fields: [x]}")},
 "pack-namespaces": {
     "sdl--pack.md": n("sdl--pack", "pack", fields={"name": "system-design-lite", "namespace": "sdl", "version": "0.1.0"}),
     "sdl--type-actor.md": n("sdl--type-actor", "type", fields={"name": "actor", "pack": "system-design-lite@0.1.0"}),
     "sdl--stray.md": n("sdl--stray", "t"),
     "lone.md": n("lone", "t", fields={"pack": "system-design-lite@0.1.0"}),
     "xyz--type-x.md": n("xyz--type-x", "type", fields={"pack": "system-design-lite@0.1.0"}),
     "foo--thing.md": n("foo--thing", "t", fields={"pack": "foo@1"}),
     "pack-old.md": n("pack-old", "pack", fields={"name": "old", "version": "1"}),
     "old2--pack.md": n("old2--pack", "pack", fields={"name": "old2", "version": "1"}),
     "pack-nameless.md": n("pack-nameless", "pack", fields={"version": "1"}),
     "weird.md": n("weird", "t", fields={"pack": 3})},
 "scratch-sandbox": {
     "main.md": n("main", "t", plane="p-main"),
     "p-main.md": n("p-main", "plane"),
     "p-draft.md": n("p-draft", "plane", fields={"scratch": "true"}),
     "p-draft2.md": n("p-draft2", "plane", fields={"scratch": "true"}),
     "p-flag.md": n("p-flag", "plane", fields={"scratch": "yes"}),
     "d1.md": n("d1", "t", plane="p-draft", links=[L("dep", "d2")]),
     "d2.md": n("d2", "t", plane="p-draft", parent="d1"),
     "d-out.md": n("d-out", "t", plane="p-draft", links=[L("dep", "main")]),
     "m-in.md": n("m-in", "t", plane="p-main", links=[L("dep", "d1")]),
     "d-other.md": n("d-other", "t", plane="p-draft2", links=[L("dep", "d1")]),
     "d-child.md": n("d-child", "t", plane="p-draft2", parent="main"),
     "f1.md": n("f1", "t", plane="p-flag"),
     "f2.md": n("f2", "t", plane="p-flag", links=[L("dep", "main")]),
     "r-all.md": rule("r-all", applies="{type: t}", check="{require_fields: [owner]}"),
     "r-id.md": rule("r-id", applies="{id: d1}", check="{require_fields: [owner]}")},
 "duplicate-definitions": {
     "a-type.md": n("a-type", "type", fields={"name": "service"}),
     "b-type.md": n("b-type", "type", fields={"name": "service"}),
     "c-type.md": n("c-type", "type", fields={"name": "other"}),
     "a-link.md": n("a-link", "link-type", fields={"name": "service"}),
     "d-type.md": n("d-type", "type", fields={"name": 3}),
     "e-type.md": n("e-type", "type", fields={"name": 3})},
 "forward-compat": {
     # A project written by a newer tool: keys, fields, types and link types this implementation has never seen.
     # It must load, check and report exactly as if they were ordinary. Additive format changes rely on this.
     "a.md": "---\nid: a\ntype: future-kind\ntitle: A\nfuture_key: ignored\nfuture_map:\n  x: 1\nfields:\n  owner: x\n  future_field:\n    nested: [1, 2]\nlinks:\n  - type: future-link\n    to: b\n    future_link_key: ignored\n---\nBody.\n",
     "b.md": n("b", "future-kind", fields={"owner": "y"}),
     "c.md": n("c", "future-kind"),
     "r.md": rule("r", applies="{type: future-kind}", check="{require_fields: [owner]}", severity="error")},
 "load-notes-ignored": {
     "a.md": n("a", "t"),
     "README.md": "# Notes\n\nNot a node.\n",
     "sub/README.md": "no frontmatter\n",
     "sub/b.md": n("b", "t"),
     ".scratch.md": "---\nnot: a node\n---\n",
     ".drafts/c.md": n("c", "t")},
})

if __name__ == "__main__":
    if CASES.exists():
        for d in CASES.iterdir():
            # only the cases this script writes: a case written by hand (derived-*, names-*, check-*) is left alone
            if d.is_dir() and d.name in cases:
                shutil.rmtree(d)
    for name, files in cases.items():
        for rel, text in files.items():
            p = CASES / name / "nodes" / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(text.encode())
    print(len(cases), "cases written")
