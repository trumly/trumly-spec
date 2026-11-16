---
id: adr--rule-owner-date
type: rule
title: An accepted decision names an owner and a date
fields:
  statement: An accepted decision names an owner and a date
  severity: warn
  applies_to:
    type: decision
    status: accepted
  check:
    require_fields:
    - owner
    - decided_on
  pack: adr@0.1.0
---
