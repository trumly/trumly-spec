---
id: gov--rule-datastore-decided
type: rule
title: Data stores have a reason
fields:
  name: Data stores have a reason
  statement: Every datastore is governed by a decision
  severity: info
  applies_to:
    type: datastore
  check:
    require_link:
      dir: in
      min: 1
      node_type: decision
      type: governs
  pack: governance@1.2.0
---
Advisory. Choosing where data lives is usually costly to reverse, so someone will ask why.

**Fix:** record a decision and link it with `governs`, or ignore this for a trivial store.
