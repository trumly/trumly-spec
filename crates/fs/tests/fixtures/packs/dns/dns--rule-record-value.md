---
id: dns--rule-record-value
type: rule
title: Every DNS record has a value
fields:
  statement: Every DNS record has a value
  severity: warn
  applies_to:
    type: dns-record
  check:
    require_fields:
    - value
  pack: dns@0.1.0
---
