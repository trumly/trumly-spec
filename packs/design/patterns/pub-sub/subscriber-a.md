---
id: subscriber-a
type: component
parent: topic
title: Subscriber A
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: topic
---
Reacts to each event. Make it safe to receive the same event twice, because delivery is at least once.
