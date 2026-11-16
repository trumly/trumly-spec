---
id: str--type-problem
type: type
title: problem
fields:
  name: problem
  plane: str--landscape
  description: Something that hurts a segment, stated from their side.
  shape: triangle
  fields:
  - name: severity
    kind: choice
    values:
    - minor
    - real
    - painful
  pack: strategy@0.1.0
---
