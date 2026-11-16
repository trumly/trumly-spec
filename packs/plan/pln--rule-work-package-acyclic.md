---
id: pln--rule-work-package-acyclic
type: rule
title: No circular work dependencies
fields:
  name: No circular work dependencies
  statement: Work packages do not depend on each other in a loop
  severity: error
  applies_to:
    type: work-package
  check:
    acyclic: depends-on
  pack: plan@1.0.0
---
**Why:** if A waits for B and B waits for A, neither can start.

**Fix:** split one work package, or remove the dependency that is not real.
