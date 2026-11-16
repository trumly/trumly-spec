---
id: sdr--rule-slo-owned
type: rule
title: Every objective is guaranteed by a component
fields:
  statement: Every objective is guaranteed by a component
  severity: error
  applies_to:
    type: slo
  check:
    require_link:
      type: guarantees
      dir: in
      node_type: component
  pack: system-design-runtime@0.1.0
---
