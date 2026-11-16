---
id: ux--type-finding
type: type
title: finding
fields:
  name: finding
  plane: ux--evidence
  description: Something research showed about a step.
  shape: triangle
  fields:
  - name: severity
    kind: choice
    values:
    - low
    - medium
    - high
  - name: status
    kind: choice
    values:
    - open
    - fixed
    - wont-fix
  pack: ux@0.1.0
---
