---
id: sdr--rule-slo-fields
type: rule
title: Every objective states a target and a window
fields:
  statement: Every objective states a target and a window
  severity: error
  applies_to:
    type: slo
  check:
    require_fields:
    - objective
    - window
  pack: system-design-runtime@0.1.0
---
