---
id: service
type: component
title: Service
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: exposes
  to: api
- type: writes
  to: store
- type: publishes
  to: events
---
Owns one business capability end to end. It is the only thing that writes its datastore; everything else asks through the API or listens to the events.
