---
id: req--type-constraint
type: type
title: constraint
fields:
  name: constraint
  plane: req--intent
  description: A limit the design must respect.
  shape: triangle
  fields:
  - name: kind
    kind: choice
    values:
    - technical
    - legal
    - budget
    - time
    - policy
  pack: requirements@0.1.0
---
