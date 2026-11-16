---
id: api--type-api-error
type: type
title: api-error
fields:
  name: api-error
  plane: api--contract
  description: A way a call can fail, and what the caller sees.
  shape: triangle
  label: error
  fields:
  - name: status_code
    kind: text
    required: true
  - name: retryable
    kind: boolean
  pack: api@0.1.0
---
