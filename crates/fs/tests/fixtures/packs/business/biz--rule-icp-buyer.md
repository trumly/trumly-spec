---
id: biz--rule-icp-buyer
type: rule
title: Every ideal customer names who buys
fields:
  statement: Every ideal customer names who buys
  severity: warn
  applies_to:
    type: icp
  check:
    require_link:
      type: bought-by
      node_type: buyer
  pack: business@0.1.0
---
