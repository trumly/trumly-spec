---
id: sd--rule-external-owner
type: rule
title: Every external dependency names an owner
fields:
  statement: Every external dependency names an owner
  severity: warn
  applies_to:
    type: external
  check:
    require_fields:
    - owner
  pack: system-design@0.3.0
---
