---
id: api--rule-endpoint-accepts-post
type: rule
title: A POST endpoint accepts a schema
fields:
  statement: A POST endpoint accepts a schema
  severity: warn
  applies_to:
    type: api-endpoint
    method: POST
  check:
    require_link:
      type: accepts
      node_type: api-schema
  pack: api@0.1.0
---
