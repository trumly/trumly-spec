---
id: subscriber-b
type: component
parent: topic
title: Subscriber B
plane: dsn--design
fields:
  kind: service
  owner: Team
  status: planned
links:
- type: subscribes
  to: topic
---
A second, independent reader. It succeeds or fails without affecting subscriber A.
