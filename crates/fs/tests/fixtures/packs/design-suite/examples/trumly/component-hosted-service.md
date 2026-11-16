---
id: component-hosted-service
type: component
title: Hosted service
plane: sd--design
parent: group-labs
fields:
  kind: service
  owner: trumly
  status: planned
links:
- type: depends-on
  to: component-core
- type: writes
  to: datastore-hosted-db
- type: consumes
  to: external-git
- type: runs-in
  to: environment-hosted-cloud
- type: satisfies
  to: requirement-hosted
---
