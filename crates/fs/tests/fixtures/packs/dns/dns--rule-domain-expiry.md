---
id: dns--rule-domain-expiry
type: rule
title: Every domain records when it expires
fields:
  statement: Every domain records when it expires
  severity: warn
  applies_to:
    type: domain
  check:
    require_fields:
    - expires
  pack: dns@0.1.0
---
