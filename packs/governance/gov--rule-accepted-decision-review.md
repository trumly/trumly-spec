---
id: gov--rule-accepted-decision-review
type: rule
title: Accepted has a review date
fields:
  name: Accepted has a review date
  statement: An accepted decision has a review-by date
  severity: warn
  applies_to:
    status: accepted
    type: decision
  check:
    require_fields:
    - review-by
  pack: governance@1.2.0
---
**Why:** circumstances change. A date keeps old decisions from becoming unquestioned habit.

**Fix:** set review-by.
