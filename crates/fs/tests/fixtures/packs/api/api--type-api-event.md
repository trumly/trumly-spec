---
id: api--type-api-event
type: type
title: api-event
fields:
  name: api-event
  plane: api--surface
  description: A message the API publishes when something happens.
  shape: chevron
  label: event
  fields:
  - name: channel
    kind: text
    required: true
  - name: delivery
    kind: choice
    values:
    - at-least-once
    - at-most-once
    - exactly-once
  pack: api@0.1.0
---
