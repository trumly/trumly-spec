---
id: def--rule-no-requirement-for-out-of-scope
type: rule
title: No specs for out-of-scope
fields:
  name: No specs for out-of-scope
  statement: No requirement derives from an out-of-scope capability
  severity: error
  applies_to:
    type: requirement
  check:
    forbid_link:
      node_fields:
        scope: out
      node_type: capability
      type: derives-from
  pack: define@1.1.0
---
**Why:** out of scope means nobody should design or build it yet.

**Fix:** drop the requirement, or change the capability's scope.
