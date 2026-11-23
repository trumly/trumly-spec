---
id: publisher
type: component
parent: topic
title: Publisher
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: publishes
  to: topic
---
Announces an event when something happens. It does not know who is listening, so adding a subscriber never changes it.
