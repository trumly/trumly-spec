---
id: biz--type-pricing-plan
type: type
title: pricing-plan
fields:
  name: pricing-plan
  plane: biz--offer
  description: What a customer pays for and what they get.
  shape: document
  fields:
  - name: price
    kind: text
    required: true
  - name: billing
    kind: choice
    values:
    - monthly
    - yearly
    - one-off
    - usage
  - name: status
    kind: choice
    values:
    - idea
    - live
    - retired
  - name: included
    kind: text
  pack: business@0.1.0
---
