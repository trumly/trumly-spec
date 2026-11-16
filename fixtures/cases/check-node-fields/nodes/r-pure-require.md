---
id: r-pure-require
type: rule
fields:
  statement: s
  applies_to: {type: comp}
  check: {require_link: {type: depends-on, node_fields: {layer: pure}}}
---
