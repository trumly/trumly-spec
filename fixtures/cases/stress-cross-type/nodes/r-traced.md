---
id: r-traced
type: rule
fields:
  statement: s
  applies_to: {type: component}
  check: {require_link: {type: implemented-by, dir: in, node_type: requirement}}
---
