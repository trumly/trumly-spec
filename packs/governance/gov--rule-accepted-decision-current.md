---
id: gov--rule-accepted-decision-current
type: rule
title: Accepted is not replaced
fields:
  name: Accepted is not replaced
  statement: An accepted decision has not been superseded
  severity: error
  applies_to:
    status: accepted
    type: decision
  check:
    forbid_link:
      dir: in
      type: supersedes
  pack: governance@1.1.0
---
**Why:** once a newer decision supersedes it, the old one should be marked superseded.

**Fix:** set the old decision's status to superseded.
