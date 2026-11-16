---
id: component-ops-api
type: component
title: Ops API service
plane: dsn--design
fields:
  kind: service
  owner: Platform team
  status: planned
links:
- type: exposes
  to: interface-ops-api
- type: writes
  to: datastore-ops-db
- type: publishes
  to: queue-job-events
- type: runs-in
  to: environment-production
- type: satisfies
  to: requirement-status-freshness
---
Serves job status to operations staff and records state changes.
