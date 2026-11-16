---
id: def--rule-metric-complete
type: rule
title: Metrics are measurable
fields:
  name: Metrics are measurable
  statement: Every metric has a unit, a target and how it is measured
  severity: error
  applies_to:
    type: metric
  check:
    require_fields:
    - unit
    - target
    - how-measured
  pack: define@1.1.0
---
**Why:** a target with no unit or method cannot be checked.

**Fix:** fill in unit, target and how-measured.
