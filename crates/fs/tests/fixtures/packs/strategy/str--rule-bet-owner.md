---
id: str--rule-bet-owner
type: rule
title: An active bet has an owner
fields:
  statement: An active bet has an owner
  severity: warn
  applies_to:
    type: bet
    status: active
  check:
    require_fields:
    - owner
  pack: strategy@0.1.0
---
