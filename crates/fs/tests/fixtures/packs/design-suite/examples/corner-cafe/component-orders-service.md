---
id: component-orders-service
type: component
title: Orders service
plane: sd--design
fields:
  kind: service
  owner: Café team
links:
- type: satisfies
  to: req-pay-online
- type: depends-on
  to: external-card-payments
- type: writes
  to: datastore-orders-db
- type: runs-in
  to: environment-cloud
---
