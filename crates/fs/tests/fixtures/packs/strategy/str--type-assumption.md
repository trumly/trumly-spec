---
id: str--type-assumption
type: type
title: assumption
fields:
  name: assumption
  plane: str--bets
  description: Something that must be true for a bet to pay off.
  shape: circle 2d
  fields:
  - name: status
    kind: choice
    values:
    - untested
    - testing
    - validated
    - invalidated
  - name: evidence
    kind: text
  pack: strategy@0.1.0
---
