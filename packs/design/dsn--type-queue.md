---
id: dsn--type-queue
type: type
title: queue
fields:
  name: queue
  plane: dsn--design
  description: 'Use for a buffer or stream that carries messages between components without the sender
    waiting: a queue, a topic or an event stream.'
  shape: pipe
  fields:
  - name: kind
    kind: choice
    required: true
    values:
    - queue
    - topic
    - stream
  - name: delivery
    kind: choice
    required: true
    values:
    - at-least-once
    - at-most-once
    - exactly-once
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.1.0
---
**Ask:** What needs to be handed from one part to another without waiting for an answer?

A queue sits between a publisher and a subscriber. Say how delivery behaves, because it decides whether consumers must cope with duplicates.

**Watch:** a queue with no publisher or no subscriber.
