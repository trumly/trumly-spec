---
id: biz--type-cost-driver
type: type
title: cost-driver
fields:
  name: cost-driver
  plane: biz--economics
  description: A cost that grows as you do.
  shape: circle 2d
  fields:
  - name: kind
    kind: choice
    values:
    - people
    - hosting
    - tools
    - marketing
    - support
    - other
  pack: business@0.1.0
---
