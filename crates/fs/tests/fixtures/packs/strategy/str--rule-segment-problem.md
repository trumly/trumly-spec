---
id: str--rule-segment-problem
type: rule
title: Every segment suffers a problem
fields:
  statement: Every segment suffers a problem
  severity: warn
  applies_to:
    type: segment
  check:
    require_link:
      type: suffers
      node_type: problem
  pack: strategy@0.1.0
---
