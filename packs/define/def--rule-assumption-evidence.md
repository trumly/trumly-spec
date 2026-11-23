---
id: def--rule-assumption-evidence
type: rule
title: Validated needs evidence
fields:
  name: Validated needs evidence
  statement: A validated assumption cites its evidence
  severity: warn
  applies_to:
    state: validated
    type: assumption
  check:
    require_fields:
    - evidence
  pack: define@1.2.0
---
**Why:** validated or invalidated needs something to point at.

**Fix:** fill in evidence.
