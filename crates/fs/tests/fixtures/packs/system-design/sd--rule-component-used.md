---
id: sd--rule-component-used
type: rule
title: Every service is used, depended on or exposes something
fields:
  statement: Every service is used, depended on or exposes something
  severity: info
  applies_to:
    type: component
    kind: service
  check:
    any_of:
    - require_link:
        type: exposes
    - require_link:
        type: uses
        dir: in
    - require_link:
        type: depends-on
        dir: in
    - require_link:
        type: consumes
        dir: in
  pack: system-design@0.3.0
---

A service nothing calls and that offers nothing may be dead.
