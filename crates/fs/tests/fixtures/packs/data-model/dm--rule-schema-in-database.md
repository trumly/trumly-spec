---
id: dm--rule-schema-in-database
type: rule
title: Schemas live in a database
fields:
  applies_to:
    type: data-schema
  check:
    require_link:
      - dir: in
        max: 1
        min: 1
        node_type: data-database
        type: data-contains
  pack: data-model@0.3.2
  severity: warn
  statement: Schemas live in a database
---
