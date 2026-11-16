---
id: sd--type-queue
type: type
title: queue
fields:
  name: queue
  plane: sd--design
  description: A buffer or stream between components.
  shape: pipe
  fields:
  - name: kind
    kind: choice
    values:
    - queue
    - topic
    - stream
  - name: delivery
    kind: choice
    values:
    - at-least-once
    - at-most-once
    - exactly-once
  pack: system-design@0.3.0
---
