---
id: sdr--rule-failure-attached
type: rule
title: Every failure mode belongs to a component
fields:
  statement: Every failure mode belongs to a component
  severity: error
  applies_to:
    type: failure-mode
  check:
    require_link:
      type: fails-by
      dir: in
      node_type: component
  pack: system-design-runtime@0.1.0
---
