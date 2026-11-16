---
id: pln--rule-risk-about
type: rule
title: Risks are about something
fields:
  name: Risks are about something
  statement: Every risk is about a node
  severity: error
  applies_to:
    type: risk
  check:
    require_link:
      min: 1
      type: about
  pack: plan@1.0.0
---
**Why:** a risk has to threaten something specific to be actionable.

**Fix:** add an `about` link to the node it threatens.
