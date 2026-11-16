---
id: def--rule-capability-has-requirement
type: rule
title: Capabilities are specified
fields:
  name: Capabilities are specified
  statement: Every in-scope capability has at least one requirement
  severity: warn
  applies_to:
    scope: in
    type: capability
  check:
    require_link:
      dir: in
      min: 1
      node_type: requirement
      type: derives-from
  pack: define@1.1.0
---
**Why:** a capability with no requirement is not yet precise enough to build or test.

**Fix:** add a requirement that `derives-from` this capability.
