---
id: dsn--rule-service-in-use
type: rule
title: Services are in use
fields:
  name: Services are in use
  statement: Every service is used, depended on or exposes something
  severity: warn
  applies_to:
    kind: service
    type: component
  check:
    any_of:
    - require_link:
        type: exposes
    - require_link:
        dir: in
        type: uses
    - require_link:
        dir: in
        type: depends-on
    - require_link:
        dir: in
        type: consumes
  pack: design@1.1.0
---
A service nothing calls and that offers nothing may be dead.

**Fix:** link it with `exposes`, `uses`, `depends-on` or `consumes`, or remove it.
