---
id: queue
type: queue
title: Work queue
plane: dsn--design
fields:
  kind: queue
  delivery: at-least-once
---
Holds work until a worker is free. The producer carries on as soon as the message is stored, and a burst waits here instead of overloading the worker. Say how long a message may wait and how many times it is retried.
