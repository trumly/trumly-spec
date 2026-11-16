---
id: ux--rule-persona-journey
type: rule
title: Every persona takes a journey
fields:
  statement: Every persona takes a journey
  severity: warn
  applies_to:
    type: persona
  check:
    require_link:
      type: takes
      node_type: journey
  pack: ux@0.1.0
---
