---
id: def--rule-volume-metric-threshold
type: rule
title: Volume metrics have a limit
fields:
  name: Volume metrics have a limit
  statement: Every volume metric has a threshold
  severity: warn
  applies_to:
    kind: volume
    type: metric
  check:
    require_fields:
    - threshold
  pack: define@1.2.0
---
**Why:** the threshold is the point at which the design should change; later change sets can watch it.

**Fix:** set threshold.
