---
id: ux--rule-journey-steps
type: rule
title: Every journey has at least two steps
fields:
  statement: Every journey has at least two steps
  severity: warn
  applies_to:
    type: journey
  check:
    require_link:
      type: has-step
      node_type: step
      min: 2
  pack: ux@0.1.0
---
