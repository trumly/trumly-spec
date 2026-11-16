---
id: work-package-status-api
type: work-package
title: Status API
plane: pln--plan
fields:
  status: planned
links:
- type: owned-by
  to: team-build-agent
- type: reviewed-by
  to: team-platform
- type: delivers
  to: component-ops-api
- type: delivers
  to: interface-ops-api
- type: part-of
  to: increment-live-status
---
Build the service that serves job status.
