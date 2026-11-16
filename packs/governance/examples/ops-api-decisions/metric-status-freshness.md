---
id: metric-status-freshness
type: metric
title: Job status age
plane: def--intent
fields:
  kind: success
  unit: seconds
  direction: lower-is-better
  target: 60
  how-measured: Age of the newest status for each job, sampled every minute
---
