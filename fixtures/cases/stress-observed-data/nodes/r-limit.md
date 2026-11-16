---
id: r-limit
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: metric, unit: ms}
  check: {compare_fields: {left: observed, op: '<=', right: limit}}
---
