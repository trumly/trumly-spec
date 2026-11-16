---
id: def--type-capability
type: type
title: capability
fields:
  name: capability
  plane: def--capabilities
  description: Use for something an actor must be able to do with the system, in plain words. It is the
    bridge between goals and precise requirements, and the place non-technical people do most of their
    thinking.
  shape: hexagon
  fields:
  - name: statement
    kind: text
    label: An actor can…
    required: true
  - name: scope
    kind: choice
    required: true
    values:
    - in
    - later
    - out
  - name: priority
    kind: choice
    required: true
    values:
    - must
    - should
    - could
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - planned
    - built
    - verified
    - dropped
  - name: accepted-by
    kind: text
    label: Accepted by (a person)
  - name: accepted-on
    kind: text
    label: Accepted on (date)
  pack: define@1.1.0
---
**Ask:** What does the actor need to be able to do? Why, which goal does it serve? Is it in scope now, later or never?

Write it as "an actor can do something". Put the normal flow in the body. Mark exclusions as scope `out` so they are explicit decisions. Requirements link up to it with `derives-from`.
