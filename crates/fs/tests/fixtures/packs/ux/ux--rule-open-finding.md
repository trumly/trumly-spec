---
id: ux--rule-open-finding
type: rule
title: A high-severity finding is not left open
fields:
  statement: A high-severity finding is not left open
  severity: warn
  applies_to:
    type: finding
    severity: high
    status: open
  check:
    compare_fields:
    - left: status
      op: '!='
      value: open
  pack: ux@0.1.0
---

Research found it and nobody acted.
