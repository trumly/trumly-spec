---
id: api--rule-event-secured
type: rule
title: Every event says how consumers are authorised
fields:
  statement: Every event says how consumers are authorised
  severity: warn
  applies_to:
    type: api-event
  check:
    require_link:
      type: secured-by
      node_type: api-auth
  pack: api@0.1.0
---
