---
id: dm--rule-concept-becomes-entity
type: rule
title: Every concept becomes an entity
fields:
  applies_to:
    type: data-concept
  check:
    require_link:
      - min: 1
        node_type: data-entity
        type: data-realised-by
  pack: data-model@0.3.2
  severity: info
  statement: Every concept becomes an entity
---
