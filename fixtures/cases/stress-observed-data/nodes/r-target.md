---
id: r-target
type: rule
fields:
  statement: s
  applies_to: {type: metric, unit: ratio}
  check: {compare_fields: {left: observed, op: '>=', right: target}}
---
