---
id: api--rule-endpoint-secured
type: rule
title: A non-public endpoint is secured
fields:
  statement: A non-public endpoint is secured
  severity: error
  applies_to:
    type: api-endpoint
    public: false
  check:
    require_link:
      type: secured-by
      node_type: api-auth
  pack: api@0.1.0
---
