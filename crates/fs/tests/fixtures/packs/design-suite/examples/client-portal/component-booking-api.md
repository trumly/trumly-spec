---
id: component-booking-api
type: component
title: Booking API
plane: sd--design
fields:
  kind: service
  owner: Mat
  status: built
links:
- type: exposes
  to: interface-booking-api
- type: writes
  to: datastore-main-db
- type: publishes
  to: queue-booking-events
- type: depends-on
  to: component-ledger
- type: runs-in
  to: env-production
- type: placed-in
  to: zone-private
- type: guarantees
  to: slo-booking-availability
- type: sized-for
  to: capacity-booking
- type: fails-by
  to: fail-db-down
- type: satisfies
  to: req-book-session
- type: consumes
  to: external-stripe
---
