---
id: adr--rule-consequence
type: rule
title: An accepted decision records a consequence
fields:
  statement: An accepted decision records a consequence
  severity: warn
  applies_to:
    type: decision
    status: accepted
  check:
    require_link:
      type: has-consequence
      node_type: consequence
  pack: adr@0.1.0
---
