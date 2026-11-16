---
id: gov--rule-consequence-belongs
type: rule
title: Consequences have a decision
fields:
  name: Consequences have a decision
  statement: Every consequence follows from a decision
  severity: warn
  applies_to:
    type: consequence
  check:
    require_link:
      dir: in
      min: 1
      node_type: decision
      type: has-consequence
  pack: governance@1.1.0
---
**Why:** a consequence only means something against the decision that caused it.

**Fix:** add `has-consequence` from the decision, or delete it.
