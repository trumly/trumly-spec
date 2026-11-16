---
id: dm--type-data-column
type: type
title: data-column
fields:
  description: A column of a table.
  fields:
  - kind: choice
    name: data_type
    required: true
    values:
    - text
    - integer
    - decimal
    - boolean
    - date
    - timestamp
    - uuid
    - json
    - binary
  - kind: boolean
    name: nullable
  - kind: boolean
    name: primary_key
  - kind: boolean
    name: foreign_key
  label: column
  name: data-column
  plane: dm--physical
  shape: circle 2d
  pack: data-model@0.3.2
---
