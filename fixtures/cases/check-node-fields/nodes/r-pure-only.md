---
id: r-pure-only
type: rule
fields:
  statement: s
  applies_to: {type: comp, layer: domain}
  check: {forbid_link: {type: depends-on, node_fields: {layer: infra}}}
---
