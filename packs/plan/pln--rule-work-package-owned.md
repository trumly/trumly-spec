---
id: pln--rule-work-package-owned
type: rule
title: Work packages have a team
fields:
  name: Work packages have a team
  statement: Every work package is owned by one team
  severity: error
  applies_to:
    type: work-package
  check:
    require_link:
      max: 1
      min: 1
      node_type: team
      type: owned-by
  pack: plan@1.0.0
---
**Why:** work with no owner does not get finished.

**Fix:** add an `owned-by` link to a team.
