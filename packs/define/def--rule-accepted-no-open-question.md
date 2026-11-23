---
id: def--rule-accepted-no-open-question
type: rule
title: Accepted has no open question
fields:
  name: Accepted has no open question
  statement: Accepted nodes have no open question about them
  severity: error
  applies_to:
    status: accepted
  check:
    forbid_link:
      dir: in
      node_fields:
        state: open
      type: about
  pack: define@1.2.0
---
**Why:** an open question means the node is not settled.

**Fix:** answer the question, or move the node back to proposed.
