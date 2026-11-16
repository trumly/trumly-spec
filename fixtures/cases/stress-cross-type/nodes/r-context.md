---
id: r-context
type: rule
fields:
  statement: s
  severity: error
  applies_to: {type: component}
  check: {require_link: {type: accesses, ancestor_type: domain, scope: other, max: 0}}
---
