---
id: sd--type-datastore
type: type
title: datastore
fields:
  name: datastore
  plane: sd--design
  description: Somewhere data lives between runs.
  shape: database
  fields:
  - name: kind
    kind: choice
    values:
    - relational
    - document
    - key-value
    - object
    - search
    - cache
    - file
  - name: owner
    kind: text
  pack: system-design@0.3.0
---
