---
id: sdz--rule-store-backup
type: rule
title: Every datastore has a backup
fields:
  statement: Every datastore has a backup
  severity: error
  applies_to:
    type: datastore
  check:
    require_link:
      type: backed-up-by
      node_type: backup
  pack: system-design-resilience@0.1.0
---
