---
id: r-chain
type: rule
fields:
  statement: s
  applies_to: {type: req}
  check: {path: [{type: built-by, node_type: comp}, {type: runs-in, node_type: env, node_plane: p1}]}
---
