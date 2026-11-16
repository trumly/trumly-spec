---
id: ep-create-booking
type: api-endpoint
title: Create a booking
plane: api--surface
fields:
  method: POST
  path: /bookings
  public: false
  idempotent: false
  status: built
links:
- type: operates-on
  to: res-booking
- type: accepts
  to: schema-booking-request
- type: returns
  to: schema-booking
- type: fails-with
  to: err-slot-taken
- type: secured-by
  to: auth-bearer
- type: satisfies
  to: req-book-session
- type: served-by
  to: component-booking-api
---
