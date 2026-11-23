---
id: def--rule-invalidated-assumption-not-relied-on
type: rule
title: Not built on false ground
fields:
  name: Not built on false ground
  statement: No accepted node relies on an invalidated assumption
  severity: error
  applies_to:
    state: invalidated
    type: assumption
  check:
    forbid_link:
      dir: in
      node_fields:
        status: accepted
      type: relies-on
  pack: define@1.2.0
---
**Why:** an accepted decision built on an invalidated assumption needs revisiting.

**Fix:** revisit the node and change its status, or replace the assumption.
