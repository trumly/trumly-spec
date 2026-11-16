---
id: sdd--rule-store-placed
type: rule
title: Every datastore is placed in a zone
fields:
  statement: Every datastore is placed in a zone
  severity: warn
  applies_to:
    type: datastore
  check:
    require_link:
      type: placed-in
      node_type: network-zone
  pack: system-design-deployment@0.1.0
---
