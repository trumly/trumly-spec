---
id: risk-job-volume-growth
type: risk
title: Job volume growth
plane: pln--plan
fields:
  likelihood: medium
  impact: high
  mitigation: Recount nightly jobs every month
  state: open
links:
- type: owned-by
  to: team-platform
- type: about
  to: assumption-job-volume
- type: arises-from
  to: assumption-job-volume
---
If job volume grows past the assumption, the database may need a bigger plan.
