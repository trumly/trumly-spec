---
id: str--rule-bet-metric
type: rule
title: Every bet is measured by a metric
fields:
  statement: Every bet is measured by a metric
  severity: error
  applies_to:
    type: bet
  check:
    require_link:
      type: measured-by
      node_type: metric
  pack: strategy@0.1.0
---

If nothing would show the bet has failed, it is a hope, not a bet.
