---
id: req--type-goal
type: type
title: goal
fields:
  name: goal
  plane: req--intent
  description: Something worth achieving, stated as an outcome.
  shape: star
  fields:
  - name: status
    kind: choice
    values:
    - proposed
    - active
    - achieved
    - dropped
  pack: requirements@0.1.0
---
