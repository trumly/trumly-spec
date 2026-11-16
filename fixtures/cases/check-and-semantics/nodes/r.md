---
id: r
type: rule
fields:
  statement: s
  applies_to: {type: svc}
  check: {require_fields: [owner], field_is: {tier: {in: [1, 2]}}}
---
