---
id: adr--rule-superseded-successor
type: rule
title: A superseded decision has a successor
fields:
  statement: A superseded decision has a successor
  severity: error
  applies_to:
    type: decision
    status: superseded
  check:
    require_link:
      type: supersedes
      dir: in
  pack: adr@0.1.0
---
