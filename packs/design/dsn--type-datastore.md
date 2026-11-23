---
id: dsn--type-datastore
type: type
title: datastore
fields:
  name: datastore
  plane: dsn--design
  description: 'Use for somewhere data lives between runs: a database, a cache, an object store, a search
    index or a file area.'
  shape: database
  fields:
  - name: kind
    kind: choice
    required: true
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
    required: true
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.1.0
---
**Ask:** Where does data live between runs? Who owns it? What would hurt to lose?

One datastore per thing with its own owner and lifecycle. The tables and fields inside it go in the data model enrichment stack.

**Watch:** a datastore nothing writes to, or that two teams both think they own.
