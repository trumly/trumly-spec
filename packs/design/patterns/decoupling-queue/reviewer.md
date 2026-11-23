---
id: reviewer
type: component
parent: queue
title: Failure handler
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: failed
---
Raises an alert for each failed message so a person looks at it. Say who is on the hook in the owner.
