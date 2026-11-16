---
id: sd--rule-component-owner
type: rule
title: Every component has an owner
fields:
  statement: Every component has an owner
  severity: warn
  applies_to:
    type: component
  check:
    require_fields:
    - owner
  pack: system-design@0.3.0
---
