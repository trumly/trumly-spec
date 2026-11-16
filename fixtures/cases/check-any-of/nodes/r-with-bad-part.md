---
id: r-with-bad-part
type: rule
fields:
  statement: s
  applies_to: {type: t}
  check: {any_of: [{require_link: {type: a}}, {nonsense: 1}]}
---
