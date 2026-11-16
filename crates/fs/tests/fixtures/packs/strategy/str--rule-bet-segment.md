---
id: str--rule-bet-segment
type: rule
title: Every bet targets a segment
fields:
  statement: Every bet targets a segment
  severity: warn
  applies_to:
    type: bet
  check:
    require_link:
      type: targets
      node_type: segment
  pack: strategy@0.1.0
---
