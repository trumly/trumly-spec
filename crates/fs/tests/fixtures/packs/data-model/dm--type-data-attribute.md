---
id: dm--type-data-attribute
type: type
title: attribute
fields:
  description: A logical attribute of an entity.
  fields:
  - kind: choice
    name: data_type
    required: true
    values:
    - text
    - number
    - boolean
    - date
    - timestamp
    - identifier
    - binary
  - kind: boolean
    name: mandatory
  name: data-attribute
  plane: dm--logical
  shape: circle 2d
  label: attribute
  pack: data-model@0.3.2
---
