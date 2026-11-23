---
id: producer
type: component
parent: queue
title: Producer
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: publishes
  to: queue
---
Puts a piece of work on the queue and carries on without waiting for the result.
