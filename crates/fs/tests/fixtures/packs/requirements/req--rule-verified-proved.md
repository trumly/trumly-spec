---
id: req--rule-verified-proved
type: rule
title: A verified requirement has a test behind its criterion
fields:
  statement: A verified requirement has a test behind its criterion
  severity: error
  applies_to:
    type: requirement
    status: verified
  check:
    path:
    - type: verified-by
      node_type: acceptance
    - type: verified-by
      node_type: test-case
  pack: requirements@0.1.0
---

Verified claims a test ran. Follow requirement, criterion, test.
