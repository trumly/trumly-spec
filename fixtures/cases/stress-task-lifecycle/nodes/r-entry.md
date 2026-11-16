---
id: r-entry
type: rule
fields:
  statement: s
  applies_to: {type: state, initial: false}
  check: {require_link: {type: transitions-to, dir: in}}
---
