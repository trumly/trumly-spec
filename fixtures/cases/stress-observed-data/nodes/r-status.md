---
id: r-status
type: rule
fields:
  statement: s
  applies_to: {type: metric}
  check: {field_is: {status: {in: [ok, degraded, down]}, observed_at: {type: string}}}
---
