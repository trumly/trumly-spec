---
id: def--rule-accepted-needs-human
type: rule
title: Accepted needs a person
fields:
  name: Accepted needs a person
  statement: Accepted nodes record accepted-by and accepted-on
  severity: error
  applies_to:
    status: accepted
  check:
    require_fields:
    - accepted-by
    - accepted-on
  pack: define@1.1.0
---
**Why:** agents propose, humans accept.

**Fix:** record who accepted it and when.
