---
id: work-package-report-job
type: work-package
title: Report job and database
plane: pln--plan
fields:
  status: planned
links:
- type: owned-by
  to: team-platform
- type: delivers
  to: component-ops-report
- type: delivers
  to: datastore-ops-db
- type: part-of
  to: increment-live-status
---
Build the database and the nightly job that reads it.
