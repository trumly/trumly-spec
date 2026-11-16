---
id: dm--rule-entity-built
type: rule
title: Every entity is built as a table
fields:
  applies_to:
    type: data-entity
  check:
    require_link:
      - min: 1
        node_type: data-table
        type: data-realised-by
  pack: data-model@0.3.2
  severity: info
  statement: Every entity is built as a table
---
