---
id: host-app
type: hostname
title: example.org
plane: dns--naming
fields:
  kind: app
links:
- type: under
  to: domain-app
- type: serves
  to: component-app
- type: serves
  to: env-production
---
