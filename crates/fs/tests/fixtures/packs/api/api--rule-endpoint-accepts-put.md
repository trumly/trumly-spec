---
id: api--rule-endpoint-accepts-put
type: rule
title: A PUT endpoint accepts a schema
fields:
  statement: A PUT endpoint accepts a schema
  severity: warn
  applies_to:
    type: api-endpoint
    method: PUT
  check:
    require_link:
      type: accepts
      node_type: api-schema
  pack: api@0.1.0
---
