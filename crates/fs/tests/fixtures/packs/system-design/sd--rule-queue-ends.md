---
id: sd--rule-queue-ends
type: rule
title: Every queue has a publisher and a subscriber
fields:
  statement: Every queue has a publisher and a subscriber
  severity: warn
  applies_to:
    type: queue
  check:
    require_link:
    - type: publishes
      dir: in
      node_type: component
    - type: subscribes
      dir: in
      node_type: component
  pack: system-design@0.3.0
---
