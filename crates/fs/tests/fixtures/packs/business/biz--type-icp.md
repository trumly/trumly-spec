---
id: biz--type-icp
type: type
title: icp
fields:
  name: icp
  plane: biz--customers
  description: 'The ideal customer: the segment you chose to win first, with what makes them look for a
    solution.'
  shape: person
  fields:
  - name: size
    kind: text
  - name: trigger
    kind: text
  - name: confidence
    kind: choice
    values:
    - low
    - medium
    - high
  pack: business@0.1.0
---
