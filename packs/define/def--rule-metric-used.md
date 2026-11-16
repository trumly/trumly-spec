---
id: def--rule-metric-used
type: rule
title: Metrics measure something
fields:
  name: Metrics measure something
  statement: Every metric measures a goal, requirement or assumption
  severity: warn
  applies_to:
    type: metric
  check:
    require_link:
      dir: in
      min: 1
      type: measured-by
  pack: define@1.1.0
---
**Why:** a metric nothing refers to is a number nobody asked for.

**Fix:** link it from the goal, requirement or assumption it measures with `measured-by`.
