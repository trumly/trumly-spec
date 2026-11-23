---
id: producer
type: component
parent: hub
title: Producer
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: publishes
  to: hub
---
Appends events to the hub. It does not wait for anyone to read them.
