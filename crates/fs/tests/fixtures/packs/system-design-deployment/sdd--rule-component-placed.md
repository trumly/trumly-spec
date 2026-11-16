---
id: sdd--rule-component-placed
type: rule
title: Every component is placed in a zone
fields:
  statement: Every component is placed in a zone
  severity: warn
  applies_to:
    type: component
  check:
    require_link:
      type: placed-in
      node_type: network-zone
  pack: system-design-deployment@0.1.0
---
