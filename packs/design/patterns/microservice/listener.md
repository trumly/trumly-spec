---
id: listener
type: component
parent: service
title: Listener
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: events
---
Another service that reacts to the events and keeps its own copy of what it needs.
