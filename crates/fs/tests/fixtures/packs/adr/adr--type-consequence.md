---
id: adr--type-consequence
type: type
title: consequence
fields:
  name: consequence
  plane: adr--decisions
  description: Something that follows from a decision, good or bad.
  shape: chevron 2d
  fields:
  - name: effect
    kind: choice
    values:
    - positive
    - negative
    - neutral
  pack: adr@0.1.0
---
