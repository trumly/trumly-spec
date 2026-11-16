---
id: str--rule-bet-assumption
type: rule
title: Every bet relies on an assumption
fields:
  statement: Every bet relies on an assumption
  severity: error
  applies_to:
    type: bet
  check:
    require_link:
      type: relies-on
      node_type: assumption
  pack: strategy@0.1.0
---

A bet with no assumption cannot be wrong, so it cannot be tested.
