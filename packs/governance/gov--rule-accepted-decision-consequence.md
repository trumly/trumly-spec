---
id: gov--rule-accepted-decision-consequence
type: rule
title: Accepted records consequences
fields:
  name: Accepted records consequences
  statement: An accepted decision records a consequence
  severity: warn
  applies_to:
    status: accepted
    type: decision
  check:
    require_link:
      min: 1
      node_type: consequence
      type: has-consequence
  pack: governance@1.1.0
---
**Why:** the consequences are what a later reader needs, especially the negative ones.

**Fix:** add a consequence with `has-consequence`.
