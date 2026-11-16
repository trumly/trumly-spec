---
id: dsn--rule-datastore-written
type: rule
title: Data stores are written to
fields:
  name: Data stores are written to
  statement: Every datastore is written by a component
  severity: warn
  applies_to:
    type: datastore
  check:
    require_link:
      dir: in
      min: 1
      node_type: component
      type: writes
  pack: design@1.0.0
---
**Why:** a store nothing writes to is either unused or has an undrawn writer.

**Fix:** add a `writes` link from the component that fills it.
