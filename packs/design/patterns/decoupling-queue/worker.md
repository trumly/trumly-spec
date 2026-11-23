---
id: worker
type: component
parent: queue
title: Worker
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: queue
- type: publishes
  to: failed
---
Takes one message at a time and does the work. Make it safe to run the same message twice, because delivery is at least once. After the retries run out it moves the message to the failed queue.
