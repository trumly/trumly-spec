---
id: sd--rule-actor-uses
type: rule
title: Every actor uses something
fields:
  statement: Every actor uses something
  severity: info
  applies_to:
    type: actor
  check:
    require_link:
      type: uses
  pack: system-design@0.3.0
---
