---
id: r-forbid-in
type: rule
fields:
  statement: s
  applies_to: {type: comp}
  check: {forbid_link: [{type: depends-on, dir: in, node_type: comp, node_fields: {layer: app}}]}
---
