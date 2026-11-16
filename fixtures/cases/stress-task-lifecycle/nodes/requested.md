---
id: requested
type: state
plane: p
fields:
  initial: true
  terminal: false
links:
  - type: transitions-to
    to: accepted
    fields: {on: accept}
  - type: transitions-to
    to: declined
    fields: {on: decline}
---
