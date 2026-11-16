---
id: r-built
type: rule
fields:
  statement: s
  applies_to: {type: req}
  check: {path: [{type: built-by, node_fields: {status: built}}]}
---
