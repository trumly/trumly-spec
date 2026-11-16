---
id: ux--type-step
type: type
title: step
fields:
  name: step
  plane: ux--flow
  description: One thing a person does or decides along a journey.
  shape: circle 2d
  fields:
  - name: kind
    kind: choice
    values:
    - action
    - decision
    - wait
  pack: ux@0.1.0
---
