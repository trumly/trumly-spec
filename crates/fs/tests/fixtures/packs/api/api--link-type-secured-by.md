---
id: api--link-type-secured-by
type: link-type
title: secured-by
fields:
  name: secured-by
  description: Callers must satisfy this to use the endpoint.
  inverse: secures
  from:
  - api-endpoint
  - api-event
  to:
  - api-auth
  pack: api@0.1.0
---
