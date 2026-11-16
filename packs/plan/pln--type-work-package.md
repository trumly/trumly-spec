---
id: pln--type-work-package
type: type
title: work-package
fields:
  name: work-package
  plane: pln--plan
  description: 'Use for a chunk of build work that one team can finish: it delivers design nodes and moves
    through planned, in-progress, built and verified.'
  shape: square
  fields:
  - name: status
    kind: choice
    required: true
    values:
    - planned
    - in-progress
    - built
    - verified
    - dropped
  - name: body
    kind: text
    label: Description
    required: true
  pack: plan@1.0.0
---
**Ask:** What is the next chunk someone can pick up and finish? What does it deliver? What must be done first?

Size it so one team can finish and verify it. It points at the design nodes it delivers; it never repeats their detail. Tickets and stories are generated from this, not stored here.

**Watch:** a work package that delivers nothing, belongs to no increment, or depends on itself in a loop.
