---
id: pln--rule-capability-enabled
type: rule
title: Musts are scheduled
fields:
  name: Musts are scheduled
  statement: Every in-scope must-have capability is enabled by an increment
  severity: warn
  applies_to:
    priority: must
    scope: in
    type: capability
  check:
    require_link:
      dir: in
      min: 1
      node_type: increment
      type: enables
  pack: plan@1.0.0
---
**Why:** an in-scope must-have that no increment enables is not in the plan.

**Fix:** add an increment with an `enables` link to the capability.
