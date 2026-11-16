---
id: r-bad-fields
type: rule
fields:
  statement: s
  applies_to: {type: comp}
  check: {require_link: {type: depends-on, node_fields: [layer]}}
---
