---
id: biz--type-unit-assumption
type: type
title: unit-assumption
fields:
  name: unit-assumption
  plane: biz--economics
  description: A number the economics rest on that you have not yet measured, or have.
  shape: shield 2d
  fields:
  - name: value
    kind: text
    required: true
  - name: review_by
    kind: text
    required: true
  - name: status
    kind: choice
    values:
    - guess
    - measured
  pack: business@0.1.0
---
