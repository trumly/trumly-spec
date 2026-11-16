---
id: sd--rule-component-runs
type: rule
title: Every component runs in an environment
fields:
  statement: Every component runs in an environment
  severity: warn
  applies_to:
    type: component
  check:
    require_link:
      type: runs-in
      node_type: environment
  pack: system-design@0.3.0
---
