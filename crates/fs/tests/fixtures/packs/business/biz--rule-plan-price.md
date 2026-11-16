---
id: biz--rule-plan-price
type: rule
title: Every pricing plan has a price
fields:
  statement: Every pricing plan has a price
  severity: error
  applies_to:
    type: pricing-plan
  check:
    require_fields:
    - price
  pack: business@0.1.0
---
