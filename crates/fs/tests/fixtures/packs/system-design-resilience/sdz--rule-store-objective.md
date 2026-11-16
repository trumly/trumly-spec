---
id: sdz--rule-store-objective
type: rule
title: Every datastore has a recovery objective
fields:
  statement: Every datastore has a recovery objective
  severity: error
  applies_to:
    type: datastore
  check:
    require_link:
      type: recovers-to
      node_type: recovery-objective
  pack: system-design-resilience@0.1.0
---
