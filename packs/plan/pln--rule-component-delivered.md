---
id: pln--rule-component-delivered
type: rule
title: Components are planned for
fields:
  name: Components are planned for
  statement: Every component is delivered by a work package
  severity: warn
  applies_to:
    type: component
  check:
    require_link:
      dir: in
      min: 1
      node_type: work-package
      type: delivers
  pack: plan@1.0.0
---
**Why:** a component no work package builds has no plan behind it.

**Fix:** add it to a work package with `delivers`.
