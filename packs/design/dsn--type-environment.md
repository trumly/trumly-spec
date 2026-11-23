---
id: dsn--type-environment
type: type
title: environment
fields:
  name: environment
  plane: dsn--design
  description: Use for a named place the system runs, such as production, staging or a developer's machine.
  shape: folder
  fields:
  - name: kind
    kind: choice
    required: true
    values:
    - production
    - staging
    - development
    - test
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.1.0
---
**Ask:** Where does it run? Production, a test copy, a developer's machine?

An environment is a named place, not a technology. Which cloud, cluster or laptop realises it is described in a hosting enrichment stack.

**Watch:** a component that runs nowhere.
