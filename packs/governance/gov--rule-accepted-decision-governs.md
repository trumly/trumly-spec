---
id: gov--rule-accepted-decision-governs
type: rule
title: Accepted decisions govern
fields:
  name: Accepted decisions govern
  statement: An accepted decision governs at least one node
  severity: warn
  applies_to:
    status: accepted
    type: decision
  check:
    require_link:
      min: 1
      type: governs
  pack: governance@1.1.0
---
**Why:** a decision that governs nothing cannot be followed or broken.

**Fix:** add `governs` links to the nodes it shapes.
