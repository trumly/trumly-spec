---
id: sdr--rule-service-slo
type: rule
title: Every service has an objective
fields:
  statement: Every service has an objective
  severity: warn
  applies_to:
    type: component
    kind: service
  check:
    require_link:
      type: guarantees
      node_type: slo
  pack: system-design-runtime@0.1.0
---
