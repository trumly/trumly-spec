---
id: adr--rule-two-options
type: rule
title: Every decision weighs at least two options
fields:
  statement: Every decision weighs at least two options
  severity: warn
  applies_to:
    type: decision
  check:
    require_link:
      type: considers
      node_type: decision-option
      min: 2
  pack: adr@0.1.0
---

With one option there was no decision.
