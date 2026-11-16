---
id: accepted
type: state
plane: p
fields:
  initial: false
  terminal: false
links:
  - type: transitions-to
    to: delivered
    fields: {on: deliver}
  - type: transitions-to
    to: cancelled
---
