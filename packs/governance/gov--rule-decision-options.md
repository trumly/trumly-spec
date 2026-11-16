---
id: gov--rule-decision-options
type: rule
title: Decisions weigh options
fields:
  name: Decisions weigh options
  statement: Every decision weighs at least two options
  severity: warn
  applies_to:
    type: decision
  check:
    require_link:
      min: 2
      node_type: decision-option
      type: considers
  pack: governance@1.1.0
---
**Why:** with one option there was no decision.

**Fix:** add the alternatives that were really considered, with `considers`.
