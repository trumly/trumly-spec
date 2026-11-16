---
id: dm--rule-index-covers-column
type: rule
title: Every index covers a column
fields:
  applies_to:
    type: data-index
  check:
    require_link:
    - min: 1
      node_type: data-column
      type: data-indexes
  severity: warn
  statement: Every index covers a column
  pack: data-model@0.3.2
---
