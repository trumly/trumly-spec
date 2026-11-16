---
id: pln--rule-increments-ordered
type: rule
title: Increments are ordered
fields:
  name: Increments are ordered
  statement: Increments do not follow each other in a loop
  severity: error
  applies_to:
    type: increment
  check:
    acyclic: follows
  pack: plan@1.0.0
---
**Why:** a loop in `follows` means there is no first slice.

**Fix:** remove the link that makes the loop.
