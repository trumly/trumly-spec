---
id: req--rule-requirement-derives
type: rule
title: Every requirement derives from a goal or a bet
fields:
  statement: Every requirement derives from a goal or a bet
  severity: warn
  applies_to:
    type: requirement
  check:
    require_link:
      type: derives-from
  pack: requirements@0.1.0
---

A requirement nobody asked for is a cost with no benefit.
