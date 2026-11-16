---
id: r
type: rule
fields:
  statement: s
  applies_to: {type: svc}
  check: {require_link: {type: runs_on, to_type: env, to_plane: p2}}
---
