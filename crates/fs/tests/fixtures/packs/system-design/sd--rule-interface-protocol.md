---
id: sd--rule-interface-protocol
type: rule
title: Every interface names its protocol
fields:
  statement: Every interface names its protocol
  severity: warn
  applies_to:
    type: interface
  check:
    require_fields:
    - protocol
  pack: system-design@0.3.0
---
