---
id: r-runs
type: rule
fields:
  statement: s
  applies_to: {type: component}
  check: {require_link: {type: runs-in, node_type: environment, node_plane: run}}
---
