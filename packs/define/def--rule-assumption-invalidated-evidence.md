---
id: def--rule-assumption-invalidated-evidence
type: rule
title: Invalidated needs evidence
fields:
  name: Invalidated needs evidence
  statement: An invalidated assumption cites its evidence
  severity: warn
  applies_to:
    state: invalidated
    type: assumption
  check:
    require_fields:
    - evidence
  pack: define@1.2.0
---
**Why:** see what showed it was wrong.

**Fix:** fill in evidence.
