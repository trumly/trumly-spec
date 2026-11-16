---
id: str--rule-problem-addressed
type: rule
title: Every problem is addressed by a bet
fields:
  statement: Every problem is addressed by a bet
  severity: info
  applies_to:
    type: problem
  check:
    require_link:
      type: addresses
      dir: in
      node_type: bet
  pack: strategy@0.1.0
---
