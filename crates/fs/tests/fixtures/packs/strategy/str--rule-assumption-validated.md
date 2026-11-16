---
id: str--rule-assumption-validated
type: rule
title: A validated assumption cites evidence
fields:
  statement: A validated assumption cites evidence
  severity: warn
  applies_to:
    type: assumption
    status: validated
  check:
    require_fields:
    - evidence
  pack: strategy@0.1.0
---
