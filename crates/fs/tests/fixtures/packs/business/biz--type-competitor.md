---
id: biz--type-competitor
type: type
title: competitor
fields:
  name: competitor
  plane: biz--position
  description: What a customer could do instead, including nothing or a spreadsheet.
  shape: triangle
  fields:
  - name: kind
    kind: choice
    values:
    - direct
    - indirect
    - do-nothing
    - in-house
  - name: strength
    kind: text
  pack: business@0.1.0
---
