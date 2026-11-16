---
id: dsn--rule-datastore-owner
type: rule
title: Data stores have an owner
fields:
  name: Data stores have an owner
  statement: Every datastore has an owner
  severity: error
  applies_to:
    type: datastore
  check:
    require_fields:
    - owner
  pack: design@1.0.0
---
**Why:** data without an owner has nobody deciding retention, backup and access.

**Fix:** set owner.
