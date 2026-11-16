---
id: component-ops-api
type: component
title: Ops API
plane: dsn--design
fields:
  kind: service
  owner: Platform
  status: planned
links:
  - type: satisfies
    to: requirement-list-jobs
  - type: writes
    to: datastore-ops-db
  - type: runs-in
    to: environment-production
---

Serves the list of jobs and their status to operations staff.
