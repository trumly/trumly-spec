---
id: gov--rule-supersedes-acyclic
type: rule
title: Supersedes never loops
fields:
  name: Supersedes never loops
  statement: Decisions never supersede each other in a loop
  severity: error
  applies_to:
    type: decision
  check:
    acyclic: supersedes
  pack: governance@1.1.0
---
**Why:** a loop means no decision is current.

**Fix:** remove the link that makes the loop.
