---
id: def--type-assumption
type: type
title: assumption
fields:
  name: assumption
  plane: def--intent
  description: Use for something believed true but not yet proven, that the design leans on. Say how it
    will be checked and who owns checking it. Volumes and user behaviour are the usual suspects.
  shape: circle 2d
  fields:
  - name: statement
    kind: text
    required: true
  - name: how-to-check
    kind: text
    required: true
  - name: owner
    kind: text
    required: true
  - name: state
    kind: choice
    required: true
    values:
    - open
    - validated
    - invalidated
  - name: evidence
    kind: text
  pack: define@1.1.0
---
**Ask:** What are we taking for granted? What would change if it were wrong? How and when will we find out, and who will?

Link the things that depend on it with `relies-on`. When it is validated or invalidated, record the evidence. An invalidated assumption must not sit under anything accepted.
