---
id: events
type: queue
parent: service
title: Service events
plane: dsn--design
fields:
  kind: topic
  delivery: at-least-once
---
What the service announces when its data changes, so other services can react without calling it.
