---
id: dm--link-type-data-realised-by
type: link-type
title: realised-by
fields:
  description: 'A higher level thing carried out one level down: concept to entity, entity to table, attribute
    to column.'
  from:
  - data-concept
  - data-entity
  - data-attribute
  inverse: realises
  label: realised-by
  name: data-realised-by
  pack: data-model@0.3.2
  to:
  - data-entity
  - data-attribute
  - data-table
  - data-column
---
