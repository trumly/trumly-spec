---
id: ux--rule-step-journey
type: rule
title: Every step belongs to a journey
fields:
  statement: Every step belongs to a journey
  severity: warn
  applies_to:
    type: step
  check:
    require_link:
      type: has-step
      dir: in
      node_type: journey
  pack: ux@0.1.0
---
