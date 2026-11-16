---
id: component-ops-report
type: component
title: Nightly report job
plane: dsn--design
fields:
  kind: job
  owner: Platform team
  status: planned
links:
- type: subscribes
  to: queue-job-events
- type: reads
  to: datastore-ops-db
- type: runs-in
  to: environment-production
- type: satisfies
  to: requirement-status-freshness
---
Builds the overnight summary from the recorded events.
