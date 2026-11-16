---
id: req--rule-built-satisfied
type: rule
title: Every built requirement is satisfied by something
fields:
  statement: Every built requirement is satisfied by something
  severity: warn
  applies_to:
    type: requirement
    status: built
  check:
    require_link:
      type: satisfies
      dir: in
  pack: requirements@0.1.0
---

Built means something delivers it: a component, an endpoint, a screen.
