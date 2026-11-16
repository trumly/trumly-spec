---
id: def--rule-requirement-derives-from
type: rule
title: Requirements have a source
fields:
  name: Requirements have a source
  statement: Every requirement derives from a capability
  severity: error
  applies_to:
    type: requirement
  check:
    require_link:
      min: 1
      node_type: capability
      type: derives-from
  pack: define@1.1.0
---
**Why:** requirements need a reason to exist.

**Fix:** add a `derives-from` link to a capability.
