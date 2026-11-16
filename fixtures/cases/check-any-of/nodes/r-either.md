---
id: r-either
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: t}
  check: {any_of: [{require_link: {type: a}}, {require_link: {type: b}}]}
---
