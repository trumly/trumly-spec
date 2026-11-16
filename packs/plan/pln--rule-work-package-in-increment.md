---
id: pln--rule-work-package-in-increment
type: rule
title: Work sits in an increment
fields:
  name: Work sits in an increment
  statement: Every work package is part of an increment
  severity: warn
  applies_to:
    type: work-package
  check:
    require_link:
      min: 1
      node_type: increment
      type: part-of
  pack: plan@1.0.0
---
**Why:** an increment is how work turns into something usable.

**Fix:** add a `part-of` link to an increment.
