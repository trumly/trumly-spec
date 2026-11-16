---
id: r-exit
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: state, terminal: false}
  check: {require_link: {type: transitions-to}}
---
