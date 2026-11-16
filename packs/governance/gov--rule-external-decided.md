---
id: gov--rule-external-decided
type: rule
title: Externals have a reason
fields:
  name: Externals have a reason
  statement: Every external is governed by a decision
  severity: info
  applies_to:
    type: external
  check:
    require_link:
      dir: in
      min: 1
      node_type: decision
      type: governs
  pack: governance@1.1.0
---
Advisory. Depending on something we do not control is a choice worth explaining.

**Fix:** record a decision and link it with `governs`.
