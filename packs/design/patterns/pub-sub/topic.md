---
id: topic
type: queue
title: Events
plane: dsn--design
fields:
  kind: topic
  delivery: at-least-once
---
The topic the publisher announces on. Every subscriber gets its own copy of each message, and a message is gone once it has been delivered, so a subscriber that was not there at the time misses it. Say what an event means in the description.
