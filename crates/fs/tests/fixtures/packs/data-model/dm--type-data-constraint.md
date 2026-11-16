---
id: dm--type-data-constraint
type: type
title: data-constraint
fields:
  description: 'A rule the database enforces on a table or columns: unique, check, not null or default.'
  fields:
  - kind: choice
    name: kind
    required: true
    values:
    - unique
    - check
    - not-null
    - default
  - kind: text
    name: expression
  label: constraint
  name: data-constraint
  plane: dm--physical
  shape: hexagon 2d
  pack: data-model@0.3.2
---
