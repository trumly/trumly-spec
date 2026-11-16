---
id: req--type-requirement
type: type
title: requirement
fields:
  name: requirement
  plane: req--requirements
  description: Something the system must do or be.
  shape: document
  fields:
  - name: kind
    kind: choice
    values:
    - functional
    - quality
  - name: priority
    kind: choice
    values:
    - must
    - should
    - could
  - name: status
    kind: choice
    values:
    - proposed
    - planned
    - built
    - verified
    - dropped
  pack: requirements@0.1.0
---
