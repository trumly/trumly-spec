---
id: str--rule-assumption-invalidated
type: rule
title: An invalidated assumption cites evidence
fields:
  statement: An invalidated assumption cites evidence
  severity: warn
  applies_to:
    type: assumption
    status: invalidated
  check:
    require_fields:
    - evidence
  pack: strategy@0.1.0
---
