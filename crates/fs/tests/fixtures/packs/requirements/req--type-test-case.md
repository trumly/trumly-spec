---
id: req--type-test-case
type: type
title: test-case
fields:
  name: test-case
  plane: req--verification
  description: A check, manual or automated, that shows an acceptance criterion holds.
  shape: octagon 2d
  fields:
  - name: kind
    kind: choice
    values:
    - unit
    - integration
    - end-to-end
    - manual
  - name: state
    kind: choice
    values:
    - planned
    - written
    - passing
    - failing
  pack: requirements@0.1.0
---
