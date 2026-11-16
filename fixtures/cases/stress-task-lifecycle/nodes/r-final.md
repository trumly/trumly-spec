---
id: r-final
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: state, terminal: true}
  check: {require_link: {type: transitions-to, max: 0}}
---
