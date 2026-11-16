---
id: r-need
type: rule
fields:
  statement: s
  applies_to: {id: [a-int, c-text]}
  check: {require_link: {type: references, same_fields: [data_type]}}
---
