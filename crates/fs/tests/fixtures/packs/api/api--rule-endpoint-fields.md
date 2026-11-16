---
id: api--rule-endpoint-fields
type: rule
title: Every endpoint states its method and path
fields:
  statement: Every endpoint states its method and path
  severity: error
  applies_to:
    type: api-endpoint
  check:
    require_fields:
    - method
    - path
  pack: api@0.1.0
---
