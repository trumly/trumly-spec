---
id: api--rule-endpoint-accepts-patch
type: rule
title: A PATCH endpoint accepts a schema
fields:
  statement: A PATCH endpoint accepts a schema
  severity: warn
  applies_to:
    type: api-endpoint
    method: PATCH
  check:
    require_link:
      type: accepts
      node_type: api-schema
  pack: api@0.1.0
---
