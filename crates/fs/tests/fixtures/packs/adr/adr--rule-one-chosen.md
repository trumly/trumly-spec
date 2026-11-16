---
id: adr--rule-one-chosen
type: rule
title: An accepted decision chose exactly one option
fields:
  statement: An accepted decision chose exactly one option
  severity: error
  applies_to:
    type: decision
    status: accepted
  check:
    require_link:
      type: considers
      node_type: decision-option
      node_fields:
        chosen: true
      min: 1
      max: 1
  pack: adr@0.1.0
---
