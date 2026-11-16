---
id: gov--type-principle
type: type
title: principle
fields:
  name: principle
  plane: gov--decisions
  description: Use for a standing rule that choices are checked against, such as "prefer managed services".
    Use a constraint for something imposed from outside and a decision for a single choice.
  shape: shield
  fields:
  - name: statement
    kind: text
    required: true
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - dropped
  - name: accepted-by
    kind: text
  - name: accepted-on
    kind: text
  - name: body
    kind: text
    label: Description
    required: true
  pack: governance@1.1.0
---
**Ask:** What rule of thumb should every future choice follow, unless there is a good reason not to?

A principle is a standing rule we adopt, unlike a constraint (imposed from outside) or a decision (one choice). A decision can follow a principle with `complies-with`, or break one with `departs-from` and say why in its description. This is how an AI assistant knows the house style before proposing anything.

**Watch:** a principle so vague that nothing could ever depart from it.
