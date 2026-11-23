---
id: def--rule-goal-has-capability
type: rule
title: Goals have a capability
fields:
  name: Goals have a capability
  statement: Every goal has at least one capability deriving from it
  severity: warn
  applies_to:
    type: goal
  check:
    require_link:
      dir: in
      min: 1
      node_type: capability
      type: derives-from
  pack: define@1.2.0
---
**Why:** a goal with no capability has no plan behind it.

**Fix:** add a capability with a `derives-from` link to the goal.
