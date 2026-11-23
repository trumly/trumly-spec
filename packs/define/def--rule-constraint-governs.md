---
id: def--rule-constraint-governs
type: rule
title: Constraints govern something
fields:
  name: Constraints govern something
  statement: Every constraint governs at least one node
  severity: warn
  applies_to:
    type: constraint
  check:
    require_link:
      min: 1
      type: governs
  pack: define@1.2.0
---
**Why:** a constraint that limits nothing is noise.

**Fix:** add a `governs` link to what it limits.
