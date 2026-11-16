---
id: sdd--rule-store-not-public
type: rule
title: A datastore is never in a public zone
fields:
  statement: A datastore is never in a public zone
  severity: error
  applies_to:
    type: datastore
  check:
    forbid_link:
      type: placed-in
      node_type: network-zone
      node_fields:
        exposure: public
  pack: system-design-deployment@0.1.0
---

Data lives behind something.
