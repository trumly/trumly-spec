---
id: r-with-sibling
type: rule
fields:
  statement: s
  applies_to: {type: t}
  check: {any_of: [{require_link: {type: b}}], require_fields: [x]}
---
