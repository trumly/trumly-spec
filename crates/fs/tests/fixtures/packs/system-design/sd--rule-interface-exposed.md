---
id: sd--rule-interface-exposed
type: rule
title: Every interface is exposed by a component
fields:
  statement: Every interface is exposed by a component
  severity: warn
  applies_to:
    type: interface
  check:
    require_link:
      type: exposes
      dir: in
      node_type: component
  pack: system-design@0.3.0
---
