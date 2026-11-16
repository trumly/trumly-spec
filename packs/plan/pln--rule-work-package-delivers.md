---
id: pln--rule-work-package-delivers
type: rule
title: Work packages deliver
fields:
  name: Work packages deliver
  statement: Every work package delivers at least one design node
  severity: error
  applies_to:
    type: work-package
  check:
    require_link:
      min: 1
      type: delivers
  pack: plan@1.0.0
---
**Why:** a work package that builds no design node is effort with no purpose.

**Fix:** add a `delivers` link to the design node it builds.
