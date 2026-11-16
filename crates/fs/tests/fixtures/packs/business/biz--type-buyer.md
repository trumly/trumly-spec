---
id: biz--type-buyer
type: type
title: buyer
fields:
  name: buyer
  plane: biz--customers
  description: Whoever decides to pay, who may not be the one who uses it.
  shape: person
  fields:
  - name: role
    kind: text
    required: true
  - name: decides
    kind: choice
    values:
    - decides
    - influences
    - signs off
  pack: business@0.1.0
---
