---
id: dsn--rule-interface-protocol
type: rule
title: Interfaces name a protocol
fields:
  name: Interfaces name a protocol
  statement: Every interface names its protocol
  severity: error
  applies_to:
    type: interface
  check:
    require_fields:
    - protocol
  pack: design@1.1.0
---
**Why:** the protocol family decides what can talk to it.

**Fix:** set protocol.
