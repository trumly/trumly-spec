---
id: gov--type-consequence
type: type
title: consequence
fields:
  name: consequence
  plane: gov--decisions
  description: Use for something that follows from a decision, good or bad.
  shape: chevron 2d
  fields:
  - name: effect
    kind: choice
    required: true
    values:
    - positive
    - negative
    - neutral
  - name: body
    kind: text
    label: Description
    required: true
  pack: governance@1.1.0
---
**Ask:** What gets easier, harder or different because of this choice?

Be honest about the negative ones. They are what a later reader needs most.

**Watch:** a decision with only positive consequences.
