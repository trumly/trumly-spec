---
id: pln--rule-increment-has-work
type: rule
title: Increments contain work
fields:
  name: Increments contain work
  statement: Every increment includes at least one work package
  severity: error
  applies_to:
    type: increment
  check:
    require_link:
      dir: in
      min: 1
      node_type: work-package
      type: part-of
  pack: plan@1.0.0
---
**Why:** an empty increment is a promise with no work behind it.

**Fix:** add work packages with `part-of`.
