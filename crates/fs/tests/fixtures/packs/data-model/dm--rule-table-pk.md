---
id: dm--rule-table-pk
type: rule
title: Every table has a primary key
fields:
  applies_to:
    type: data-table
  check:
    require_link:
      - min: 1
        node_fields:
          primary_key: true
        node_type: data-column
        type: data-contains
  pack: data-model@0.3.2
  severity: warn
  statement: Every table has a primary key
---
