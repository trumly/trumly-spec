---
id: dns--type-hostname
type: type
title: hostname
fields:
  name: hostname
  plane: dns--naming
  description: A name that resolves to something, under a domain.
  shape: circle 2d
  fields:
  - name: kind
    kind: choice
    values:
    - site
    - app
    - api
    - mail
    - other
    required: true
  pack: dns@0.1.0
---
