---
id: dm--link-type-data-contains
type: link-type
title: contains
fields:
  description: 'Something sits inside another: a database in a server, a column in a table.'
  from:
  - data-db-server
  - data-database
  - data-schema
  - data-table
  - data-entity
  inverse: contained-by
  label: contains
  name: data-contains
  pack: data-model@0.3.2
  to:
  - data-database
  - data-schema
  - data-table
  - data-view
  - data-proc
  - data-column
  - data-index
  - data-constraint
  - data-attribute
---
