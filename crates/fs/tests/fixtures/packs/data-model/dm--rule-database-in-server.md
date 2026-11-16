---
id: dm--rule-database-in-server
type: rule
title: Databases run in servers
fields:
  applies_to:
    type: data-database
  check:
    require_link:
      - dir: in
        min: 1
        node_type: data-db-server
        type: data-contains
  pack: data-model@0.3.2
  severity: warn
  statement: Databases run in servers
---
