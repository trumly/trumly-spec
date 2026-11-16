---
id: biz--rule-bet-icp
type: rule
title: Every active bet is aimed at an ideal customer
fields:
  statement: Every active bet is aimed at an ideal customer
  severity: warn
  applies_to:
    type: bet
    status: active
  check:
    require_link:
      type: aims-at
      node_type: icp
  pack: business@0.1.0
---
