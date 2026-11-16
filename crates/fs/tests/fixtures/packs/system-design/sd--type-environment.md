---
id: sd--type-environment
type: type
title: environment
fields:
  name: environment
  plane: sd--runtime
  description: A place the system runs.
  shape: folder
  fields:
  - name: kind
    kind: choice
    values:
    - production
    - staging
    - development
    - test
  pack: system-design@0.3.0
---
