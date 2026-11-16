---
id: str
type: rule
fields:
  statement: s
  enabled: 'false'
  applies_to: {type: svc}
  check: {require_link: {type: needs}}
---
