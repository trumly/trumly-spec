---
id: api--rule-endpoint-resource
type: rule
title: Every endpoint operates on a resource
fields:
  statement: Every endpoint operates on a resource
  severity: warn
  applies_to:
    type: api-endpoint
  check:
    require_link:
      type: operates-on
      node_type: api-resource
  pack: api@0.1.0
---
