---
id: biz--type-channel
type: type
title: channel
fields:
  name: channel
  plane: biz--reach
  description: A place customers hear about you.
  shape: hexagon 2d
  fields:
  - name: kind
    kind: choice
    values:
    - owned
    - paid
    - earned
    - partner
    - outbound
  - name: cost
    kind: text
  pack: business@0.1.0
---
