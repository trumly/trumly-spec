---
id: gov--rule-superseded-decision-successor
type: rule
title: Superseded has a successor
fields:
  name: Superseded has a successor
  statement: A superseded decision has a newer decision that supersedes it
  severity: error
  applies_to:
    status: superseded
    type: decision
  check:
    require_link:
      dir: in
      min: 1
      type: supersedes
  pack: governance@1.1.0
---
**Why:** the reader needs to know what replaced it.

**Fix:** add `supersedes` from the newer decision.
