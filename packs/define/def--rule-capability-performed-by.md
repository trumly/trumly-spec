---
id: def--rule-capability-performed-by
type: rule
title: Capabilities have an actor
fields:
  name: Capabilities have an actor
  statement: Every capability is performed by an actor
  severity: error
  applies_to:
    type: capability
  check:
    require_link:
      min: 1
      node_type: actor
      type: performed-by
  pack: define@1.2.0
---
**Why:** if nobody uses it, it is not a capability.

**Fix:** add a `performed-by` link to an actor.
