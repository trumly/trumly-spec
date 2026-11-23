---
id: def--rule-quality-requirement-measured
type: rule
title: Quality needs a metric
fields:
  name: Quality needs a metric
  statement: Every quality requirement is measured by a metric
  severity: error
  applies_to:
    kind: quality
    type: requirement
  check:
    require_link:
      min: 1
      node_type: metric
      type: measured-by
  pack: define@1.2.0
---
**Why:** "fast" and "secure" mean nothing until there is a number.

**Fix:** add a metric and link it with `measured-by`.
