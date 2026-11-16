---
id: dm--rule-column-in-table
type: rule
title: Columns belong to one table
fields:
  applies_to:
    type: data-column
  check:
    require_link:
      - dir: in
        max: 1
        min: 1
        node_type: data-table
        type: data-contains
  pack: data-model@0.3.2
  severity: warn
  statement: Columns belong to one table
---
