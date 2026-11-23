---
id: gov--type-decision-option
type: type
title: decision-option
fields:
  name: decision-option
  plane: gov--decisions
  description: Use for one way a decision could go. Mark the one that was picked as chosen.
  shape: hexagon 2d
  fields:
  - name: chosen
    kind: boolean
  - name: summary
    kind: text
  - name: body
    kind: text
    label: Description
    required: true
  pack: governance@1.2.0
---
**Ask:** What else could we have done? What is good and bad about each?

Create options per decision (do not share one option node between decisions, because `chosen` belongs to a single decision). Put the pros and cons in the description.

**Watch:** a decision where every option looks like a straw man.
