---
id: r
type: rule
fields:
  statement: s
  applies_to: {type: svc}
  check: {require_link: [{type: owns, node_type: team, node_plane: q}, {type: uses, min: 2}, {type: ok}]}
---
