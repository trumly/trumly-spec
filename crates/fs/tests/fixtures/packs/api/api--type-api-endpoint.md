---
id: api--type-api-endpoint
type: type
title: api-endpoint
fields:
  name: api-endpoint
  plane: api--surface
  description: One operation a caller can make.
  shape: hexagon
  label: endpoint
  fields:
  - name: method
    kind: choice
    values:
    - GET
    - POST
    - PUT
    - PATCH
    - DELETE
    required: true
  - name: path
    kind: text
    required: true
  - name: public
    kind: boolean
  - name: idempotent
    kind: boolean
  - name: status
    kind: choice
    values:
    - proposed
    - built
    - deprecated
  pack: api@0.1.0
---
