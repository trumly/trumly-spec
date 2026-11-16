---
id: r-and-inside
type: rule
fields:
  statement: s
  applies_to: {type: t}
  check: {any_of: [{require_fields: [x], require_link: {type: a}}, {require_link: {type: b}}]}
---
