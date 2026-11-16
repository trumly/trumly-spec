---
id: api--rule-error-code
type: rule
title: Every error states its status code
fields:
  statement: Every error states its status code
  severity: error
  applies_to:
    type: api-error
  check:
    require_fields:
    - status_code
  pack: api@0.1.0
---
