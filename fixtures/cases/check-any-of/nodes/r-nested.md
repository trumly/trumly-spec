---
id: r-nested
type: rule
fields:
  statement: s
  applies_to: {type: t}
  check: {any_of: [{any_of: [{require_link: {type: a}}, {require_link: {type: b}}]}, {require_fields: [x]}]}
---
