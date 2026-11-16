---
id: sdr--rule-slo-alert
type: rule
title: Every objective names the alert that fires when it is missed
fields:
  statement: Every objective names the alert that fires when it is missed
  severity: error
  applies_to:
    type: slo
  check:
    require_fields:
    - alert
  pack: system-design-runtime@0.1.0
---
