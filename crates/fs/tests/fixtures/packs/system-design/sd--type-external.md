---
id: sd--type-external
type: type
title: external
fields:
  name: external
  plane: sd--context
  description: Something we depend on but do not own.
  shape: cloud
  fields:
  - name: owner
    kind: text
    required: true
  pack: system-design@0.3.0
---
