---
id: adr--rule-accepted-current
type: rule
title: An accepted decision has not been superseded
fields:
  statement: An accepted decision has not been superseded
  severity: error
  applies_to:
    type: decision
    status: accepted
  check:
    forbid_link:
      type: supersedes
      dir: in
  pack: adr@0.1.0
---
