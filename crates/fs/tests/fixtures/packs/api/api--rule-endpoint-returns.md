---
id: api--rule-endpoint-returns
type: rule
title: Every endpoint returns a schema
fields:
  statement: Every endpoint returns a schema
  severity: warn
  applies_to:
    type: api-endpoint
  check:
    require_link:
      type: returns
      node_type: api-schema
  pack: api@0.1.0
---
