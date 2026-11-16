---
id: biz--type-offer
type: type
title: offer
fields:
  name: offer
  plane: biz--offer
  description: 'A deal around a plan: a founding-member price, a trial, a pilot.'
  shape: diamond
  fields:
  - name: kind
    kind: choice
    values:
    - founding-member
    - trial
    - discount
    - pilot
  - name: ends
    kind: text
  - name: limit
    kind: text
  pack: business@0.1.0
---
