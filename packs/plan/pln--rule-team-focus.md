---
id: pln--rule-team-focus
type: rule
title: Teams state their focus
fields:
  name: Teams state their focus
  statement: Every team states its focus
  severity: error
  applies_to:
    type: team
  check:
    require_fields:
    - focus
  pack: plan@1.0.0
---
**Why:** for an agent, the focus is what keeps its work reviewable.

**Fix:** set focus.
