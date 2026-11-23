---
id: dsn--rule-queue-ends
type: rule
title: Queues have both ends
fields:
  name: Queues have both ends
  statement: Every queue has a publisher and a subscriber
  severity: warn
  applies_to:
    type: queue
  check:
    require_link:
    - dir: in
      node_type: component
      type: publishes
    - dir: in
      node_type: component
      type: subscribes
  pack: design@1.1.0
---
**Why:** a queue nobody writes to never delivers; one nobody reads from fills up.

**Fix:** add `publishes` from the sending component and `subscribes` from the receiving one.
