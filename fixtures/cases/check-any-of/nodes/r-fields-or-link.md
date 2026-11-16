---
id: r-fields-or-link
type: rule
fields:
  statement: s
  applies_to: {type: t}
  check: {any_of: [{require_fields: [x]}, {require_link: {type: a}}]}
---
