---
id: r-event
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: state}
  check: {require_link_fields: {type: transitions-to, fields: [on]}}
---
