---
id: def--rule-functional-requirement-verified
type: rule
title: Functional has a criterion
fields:
  name: Functional has a criterion
  statement: Every functional requirement is verified by an acceptance criterion
  severity: error
  applies_to:
    kind: functional
    type: requirement
  check:
    require_link:
      min: 1
      node_type: acceptance-criterion
      type: verified-by
  pack: define@1.2.0
---
**Why:** if you cannot say how you would prove it, it is not clear enough.

**Fix:** add an acceptance criterion and link it with `verified-by`.
