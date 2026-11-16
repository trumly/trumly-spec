---
id: ux--rule-finding-step
type: rule
title: Every finding is tied to a step
fields:
  statement: Every finding is tied to a step
  severity: error
  applies_to:
    type: finding
  check:
    require_link:
      type: observed-at
      node_type: step
  pack: ux@0.1.0
---
