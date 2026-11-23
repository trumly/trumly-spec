---
id: hub
type: queue
title: Event hub
plane: dsn--design
fields:
  kind: stream
  delivery: at-least-once
---
An ordered, durable log. Events stay for a set time, so a new consumer can start from the beginning and a failed one can read again. Say how long events are kept and what decides their order (a key, such as the job id) in the description.
