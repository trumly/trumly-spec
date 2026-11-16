---
id: api--rule-event-payload
type: rule
title: Every event carries a schema
fields:
  statement: Every event carries a schema
  severity: error
  applies_to:
    type: api-event
  check:
    require_link:
      type: payload
      node_type: api-schema
  pack: api@0.1.0
---
