---
id: consumer-a
type: component
parent: hub
title: Consumer A
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: hub
---
Reads the whole stream at its own pace and remembers how far it has got. Make it safe to read an event twice.
