---
id: r-same
type: rule
fields:
  statement: s
  applies_to: {type: col}
  check: {require_link: {type: references, same_fields: [data_type], min: 0, max: 0}}
---
