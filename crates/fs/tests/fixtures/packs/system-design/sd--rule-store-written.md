---
id: sd--rule-store-written
type: rule
title: Every datastore is written by a component
fields:
  statement: Every datastore is written by a component
  severity: warn
  applies_to:
    type: datastore
  check:
    require_link:
      type: writes
      dir: in
      node_type: component
  pack: system-design@0.3.0
---
