---
id: dsn--rule-external-owner
type: rule
title: Externals have an owner
fields:
  name: Externals have an owner
  statement: Every external names who owns it
  severity: error
  applies_to:
    type: external
  check:
    require_fields:
    - owner
  pack: design@1.0.0
---
**Why:** when it fails you need someone to ask.

**Fix:** set owner.
