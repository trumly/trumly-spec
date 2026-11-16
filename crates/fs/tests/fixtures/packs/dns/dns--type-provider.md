---
id: dns--type-provider
type: type
title: provider
fields:
  name: provider
  plane: dns--naming
  description: A company that supplies registration, DNS, hosting, delivery or mail.
  shape: triangle
  fields:
  - name: role
    kind: choice
    values:
    - registrar
    - dns
    - hosting
    - cdn
    - email
    - other
    required: true
  pack: dns@0.1.0
---
