---
id: biz--type-revenue-stream
type: type
title: revenue-stream
fields:
  name: revenue-stream
  plane: biz--economics
  description: A way money comes in.
  shape: circle 2d
  fields:
  - name: model
    kind: choice
    values:
    - subscription
    - usage
    - one-off
    - services
  pack: business@0.1.0
---
