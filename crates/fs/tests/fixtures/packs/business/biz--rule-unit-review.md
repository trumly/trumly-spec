---
id: biz--rule-unit-review
type: rule
title: Every unit assumption has a value and a review date
fields:
  statement: Every unit assumption has a value and a review date
  severity: error
  applies_to:
    type: unit-assumption
  check:
    require_fields:
    - value
    - review_by
  pack: business@0.1.0
---

A guess with no review date becomes a fact by neglect.
