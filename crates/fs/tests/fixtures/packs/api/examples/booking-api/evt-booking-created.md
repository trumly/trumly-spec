---
id: evt-booking-created
type: api-event
title: Booking created
plane: api--surface
fields:
  channel: bookings.created
  delivery: at-least-once
links:
- type: payload
  to: schema-booking
- type: secured-by
  to: auth-bearer
---
