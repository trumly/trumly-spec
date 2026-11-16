---
id: adr-use-postgres
type: decision
title: Use Postgres for the main store
plane: adr--decisions
fields:
  status: accepted
  owner: Mat
  decided_on: '2026-10-01'
links:
- type: considers
  to: option-postgres
- type: considers
  to: option-dynamodb
- type: has-consequence
  to: consequence-simple-ops
- type: governs
  to: datastore-main-db
- type: supersedes
  to: adr-use-sqlite
---
