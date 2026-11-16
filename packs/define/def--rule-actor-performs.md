---
id: def--rule-actor-performs
type: rule
title: Actors do something
fields:
  name: Actors do something
  statement: Every actor performs at least one capability
  severity: warn
  applies_to:
    type: actor
  check:
    require_link:
      dir: in
      min: 1
      node_type: capability
      type: performed-by
  pack: define@1.1.0
---
**Why:** a role that performs no capability has no reason to be in the design.

**Fix:** link a capability to it with `performed-by`, or remove the actor.
