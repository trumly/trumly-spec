---
id: sdd--rule-zone-region
type: rule
title: Every network zone is in a region
fields:
  statement: Every network zone is in a region
  severity: warn
  applies_to:
    type: network-zone
  check:
    require_link:
      type: in-region
      node_type: region
  pack: system-design-deployment@0.1.0
---
