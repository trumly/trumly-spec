---
id: str--rule-metric-target
type: rule
title: Every metric has a target and a review date
fields:
  statement: Every metric has a target and a review date
  severity: error
  applies_to:
    type: metric
  check:
    require_fields:
    - target
    - review_by
  pack: strategy@0.1.0
---
