---
id: def--rule-assumption-checkable
type: rule
title: Assumptions can be checked
fields:
  name: Assumptions can be checked
  statement: Every assumption has how-to-check and an owner
  severity: error
  applies_to:
    type: assumption
  check:
    require_fields:
    - how-to-check
    - owner
  pack: define@1.2.0
---
**Why:** an assumption nobody owns or tests quietly becomes a fact.

**Fix:** fill in how-to-check and owner.
