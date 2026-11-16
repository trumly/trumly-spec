---
id: biz--rule-channel-reaches
type: rule
title: Every channel reaches an ideal customer
fields:
  statement: Every channel reaches an ideal customer
  severity: warn
  applies_to:
    type: channel
  check:
    require_link:
      type: reaches
      node_type: icp
  pack: business@0.1.0
---
