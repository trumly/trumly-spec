---
id: biz--rule-revenue-plan
type: rule
title: Every revenue stream is earned from a plan
fields:
  statement: Every revenue stream is earned from a plan
  severity: warn
  applies_to:
    type: revenue-stream
  check:
    require_link:
      type: earned-from
      node_type: pricing-plan
  pack: business@0.1.0
---
