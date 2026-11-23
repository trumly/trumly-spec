---
id: def--type-acceptance-criterion
type: type
title: acceptance-criterion
fields:
  name: acceptance-criterion
  plane: def--requirements
  description: 'Use for the concrete check that proves a requirement is met: what is set up, what happens
    and what must result. It is the contract between the people who ask and the people who build.'
  shape: square
  fields:
  - name: given
    kind: text
  - name: when
    kind: text
  - name: then
    kind: text
    required: true
  - name: method
    kind: choice
    required: true
    values:
    - test
    - inspection
    - demonstration
    - analysis
  pack: define@1.2.0
---
**Ask:** How would we show this is met? What would we set up, what would we do, what must we see?

Given / when / then keeps it concrete. Choose the method honestly: automated or manual **test**, **inspection** of an artefact, a live **demonstration**, or **analysis** (a calculation or review). Link it from its requirement with `verified-by`.
