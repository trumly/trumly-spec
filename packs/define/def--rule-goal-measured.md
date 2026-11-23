---
id: def--rule-goal-measured
type: rule
title: Goals have a metric
fields:
  name: Goals have a metric
  statement: Every goal is measured by a metric
  severity: error
  applies_to:
    type: goal
  check:
    require_link:
      min: 1
      node_type: metric
      type: measured-by
  pack: define@1.2.0
---
**Why:** a goal nobody can measure cannot be seen to succeed or fail.

**Fix:** add a `measured-by` link to a metric.
