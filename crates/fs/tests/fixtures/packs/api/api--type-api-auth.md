---
id: api--type-api-auth
type: type
title: api-auth
fields:
  name: api-auth
  plane: api--contract
  description: How a caller proves who they are.
  shape: shield
  label: auth
  fields:
  - name: scheme
    kind: choice
    values:
    - api-key
    - bearer
    - oauth2
    - mtls
    - session
    required: true
  - name: scopes
    kind: text
  pack: api@0.1.0
---
