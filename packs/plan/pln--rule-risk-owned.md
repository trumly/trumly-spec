---
id: pln--rule-risk-owned
type: rule
title: Risks have an owner
fields:
  name: Risks have an owner
  statement: Every risk is owned by a team
  severity: error
  applies_to:
    type: risk
  check:
    require_link:
      min: 1
      node_type: team
      type: owned-by
  pack: plan@1.0.0
---
**Why:** a risk nobody owns is a risk nobody watches.

**Fix:** add an `owned-by` link to a team.
