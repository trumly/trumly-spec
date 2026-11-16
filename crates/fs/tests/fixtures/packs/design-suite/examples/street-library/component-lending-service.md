---
id: component-lending-service
type: component
title: Lending service
plane: sd--design
fields:
  kind: service
  owner: Street group
links:
- type: writes
  to: datastore-lending-db
- type: depends-on
  to: component-reminder-job
- type: runs-in
  to: environment-cloud
---
