---
id: dsn--rule-component-owner
type: rule
title: Components have an owner
fields:
  name: Components have an owner
  statement: Every component has an owner
  severity: error
  applies_to:
    type: component
  check:
    require_fields:
    - owner
  pack: design@1.1.0
---
**Why:** someone has to answer for it when it breaks or needs to change.

**Fix:** set owner.
