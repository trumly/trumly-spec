---
id: def--rule-capability-derives-from-goal
type: rule
title: Capabilities trace to a goal
fields:
  name: Capabilities trace to a goal
  statement: Every capability derives from a goal
  severity: error
  applies_to:
    type: capability
  check:
    require_link:
      min: 1
      node_type: goal
      type: derives-from
  pack: define@1.1.0
---
**Why:** capabilities without a goal are scope creep.

**Fix:** add a `derives-from` link to the goal it serves.
