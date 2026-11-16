---
id: ep-list-slots
type: api-endpoint
title: List free slots
plane: api--surface
fields:
  method: GET
  path: /slots
  public: true
  idempotent: true
  status: built
links:
- type: operates-on
  to: res-booking
- type: returns
  to: schema-slot-list
- type: fails-with
  to: err-bad-request
- type: satisfies
  to: req-book-session
- type: served-by
  to: component-booking-api
---
