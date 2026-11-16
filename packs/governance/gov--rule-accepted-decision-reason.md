---
id: gov--rule-accepted-decision-reason
type: rule
title: Accepted says what forced it
fields:
  name: Accepted says what forced it
  statement: An accepted decision derives from a requirement, constraint or principle
  severity: warn
  applies_to:
    status: accepted
    type: decision
  check:
    require_link:
      min: 1
      type: derives-from
  pack: governance@1.1.0
---
**Why:** a decision with no driver is a preference. Link it to the requirement, constraint or principle behind it.

**Fix:** add a `derives-from` link.
