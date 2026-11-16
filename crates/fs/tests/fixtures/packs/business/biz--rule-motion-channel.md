---
id: biz--rule-motion-channel
type: rule
title: Every motion runs through a channel
fields:
  statement: Every motion runs through a channel
  severity: warn
  applies_to:
    type: motion
  check:
    require_link:
      type: runs-through
      node_type: channel
  pack: business@0.1.0
---
