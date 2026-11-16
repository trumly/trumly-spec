---
id: component-ledger
type: component
title: Credit ledger
plane: sd--design
fields:
  kind: service
  owner: Mat
  status: built
links:
- type: writes
  to: datastore-main-db
- type: subscribes
  to: queue-booking-events
- type: runs-in
  to: env-production
- type: satisfies
  to: req-credit-ledger
---
