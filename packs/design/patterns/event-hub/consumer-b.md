---
id: consumer-b
type: component
parent: hub
title: Consumer B
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: hub
---
An independent reader with its own position, for example one that rebuilds a report from the start while another keeps a live view.
