---
id: def--rule-quality-requirement-attribute
type: rule
title: Quality names its attribute
fields:
  name: Quality names its attribute
  statement: Every quality requirement names a quality attribute
  severity: error
  applies_to:
    kind: quality
    type: requirement
  check:
    require_fields:
    - quality-attribute
  pack: define@1.2.0
---
**Why:** the attribute (performance, security, reliability...) tells readers which design decisions it will shape.

**Fix:** set quality-attribute.
