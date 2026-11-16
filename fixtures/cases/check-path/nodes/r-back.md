---
id: r-back
type: rule
fields:
  statement: s
  applies_to: {type: env}
  check: {path: [{type: runs-in, dir: in}, {type: built-by, dir: in, node_type: req}]}
---
