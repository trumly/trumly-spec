---
id: str--type-bet
type: type
title: bet
fields:
  name: bet
  plane: str--bets
  description: A choice to spend effort on one thing instead of another, stated so it can be proven wrong.
  shape: diamond
  fields:
  - name: status
    kind: choice
    values:
    - proposed
    - active
    - won
    - lost
    - dropped
  - name: confidence
    kind: choice
    values:
    - low
    - medium
    - high
  - name: owner
    kind: text
  - name: horizon
    kind: text
  pack: strategy@0.1.0
---
