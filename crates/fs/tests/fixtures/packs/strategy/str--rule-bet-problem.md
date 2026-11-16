---
id: str--rule-bet-problem
type: rule
title: Every bet addresses a problem
fields:
  statement: Every bet addresses a problem
  severity: warn
  applies_to:
    type: bet
  check:
    require_link:
      type: addresses
      node_type: problem
  pack: strategy@0.1.0
---
