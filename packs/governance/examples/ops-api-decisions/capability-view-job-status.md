---
id: capability-view-job-status
type: capability
title: View job status
plane: def--capabilities
fields:
  statement: view the current status of every job
  scope: in
  priority: must
  status: proposed
links:
- type: performed-by
  to: actor-ops-staff
- type: derives-from
  to: goal-timely-visibility
- type: refers-to
  to: concept-job
- type: relies-on
  to: assumption-job-volume
---
