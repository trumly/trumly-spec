---
id: sdz--rule-objective-fields
type: rule
title: Every recovery objective states time and data loss
fields:
  statement: Every recovery objective states time and data loss
  severity: error
  applies_to:
    type: recovery-objective
  check:
    require_fields:
    - rto
    - rpo
  pack: system-design-resilience@0.1.0
---
