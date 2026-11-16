---
id: dns--type-domain
type: type
title: domain
fields:
  name: domain
  plane: dns--naming
  description: A registered name, the root that hostnames sit under.
  shape: hexagon 2d
  fields:
  - name: registrar
    kind: text
  - name: expires
    kind: text
    label: Expires (YYYY-MM-DD)
  pack: dns@0.1.0
---
