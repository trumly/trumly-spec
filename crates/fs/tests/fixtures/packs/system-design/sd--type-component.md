---
id: sd--type-component
type: type
title: component
fields:
  name: component
  plane: sd--design
  description: A part of the system that does work.
  shape: square
  fields:
  - name: kind
    kind: choice
    values:
    - service
    - app
    - job
    - library
    - function
  - name: owner
    kind: text
  - name: status
    kind: choice
    values:
    - proposed
    - planned
    - built
  pack: system-design@0.3.0
---
