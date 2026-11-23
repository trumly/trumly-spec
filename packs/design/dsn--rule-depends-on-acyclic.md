---
id: dsn--rule-depends-on-acyclic
type: rule
title: No circular dependencies
fields:
  name: No circular dependencies
  statement: Dependencies do not form a loop
  severity: error
  applies_to:
    type: component
  check:
    acyclic: depends-on
  pack: design@1.1.0
---
**Why:** if A needs B and B needs A, neither can start first.

**Fix:** break the loop, usually by introducing a queue or moving shared logic into a third component.
