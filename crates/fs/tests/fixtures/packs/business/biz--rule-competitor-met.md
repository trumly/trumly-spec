---
id: biz--rule-competitor-met
type: rule
title: Every competitor is met by a differentiator
fields:
  statement: Every competitor is met by a differentiator
  severity: warn
  applies_to:
    type: competitor
  check:
    require_link:
      type: beats
      dir: in
      node_type: differentiator
  pack: business@0.1.0
---

If you have no answer to an alternative, say so in the model before a customer asks.
