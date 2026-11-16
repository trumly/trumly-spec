---
id: biz--rule-icp-reached
type: rule
title: Every ideal customer is reached by a channel
fields:
  statement: Every ideal customer is reached by a channel
  severity: info
  applies_to:
    type: icp
  check:
    require_link:
      type: reaches
      dir: in
      node_type: channel
  pack: business@0.1.0
---
