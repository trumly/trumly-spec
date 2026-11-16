---
id: declined
type: state
plane: p
fields:
  initial: false
  terminal: true
links:
  - type: transitions-to
    to: requested
    fields: {on: retry}
---
