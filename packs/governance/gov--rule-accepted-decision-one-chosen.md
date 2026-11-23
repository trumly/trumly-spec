---
id: gov--rule-accepted-decision-one-chosen
type: rule
title: Accepted decides one option
fields:
  name: Accepted decides one option
  statement: An accepted decision chose exactly one option
  severity: error
  applies_to:
    status: accepted
    type: decision
  check:
    require_link:
      max: 1
      min: 1
      node_fields:
        chosen: true
      node_type: decision-option
      type: considers
  pack: governance@1.2.0
---
**Why:** accepted means the choice has been made.

**Fix:** set `chosen` on exactly one of the options.
