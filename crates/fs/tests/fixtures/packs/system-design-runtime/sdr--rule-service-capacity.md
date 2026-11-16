---
id: sdr--rule-service-capacity
type: rule
title: Every service says what it is sized for
fields:
  statement: Every service says what it is sized for
  severity: warn
  applies_to:
    type: component
    kind: service
  check:
    require_link:
      type: sized-for
      node_type: capacity
  pack: system-design-runtime@0.1.0
---
