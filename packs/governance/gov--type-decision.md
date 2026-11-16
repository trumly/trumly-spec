---
id: gov--type-decision
type: type
title: decision
fields:
  name: decision
  plane: gov--decisions
  description: Use to record a choice that shaped the design, with the options weighed and what follows
    from it. Record one whenever a choice would be hard or costly to reverse, or someone would ask why.
  shape: diamond
  fields:
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - rejected
    - superseded
    - deprecated
  - name: accepted-by
    kind: text
  - name: accepted-on
    kind: text
  - name: review-by
    kind: text
  - name: body
    kind: text
    label: Description
    required: true
  pack: governance@1.1.0
---
**Ask:** What was chosen, and what else was on the table? Why this one? What does it commit us to? What would make us revisit it?

Put the context and reasoning in the description. Weigh at least two options, with exactly one chosen, and record what follows. Link it to what forced it (`derives-from` a requirement, constraint or principle), what it relies on (`relies-on` an assumption), and what it shapes (`governs`). An agent may propose; a human accepts. Keep the title to a short label.

**Watch:** a decision with only one option, one that governs nothing, or one still resting on an assumption that has been invalidated.
