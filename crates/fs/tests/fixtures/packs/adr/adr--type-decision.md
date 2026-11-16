---
id: adr--type-decision
type: type
title: decision
fields:
  name: decision
  plane: adr--decisions
  description: A choice that shaped the design. The context and the reasoning go in the body.
  shape: diamond
  fields:
  - name: status
    kind: choice
    values:
    - proposed
    - accepted
    - rejected
    - superseded
    - deprecated
    required: true
  - name: owner
    kind: text
  - name: decided_on
    kind: text
  pack: adr@0.1.0
---
