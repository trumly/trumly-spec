---
id: dsn--rule-component-satisfies
type: rule
title: Components serve a need
fields:
  name: Components serve a need
  statement: Every component satisfies at least one requirement
  severity: warn
  applies_to:
    type: component
  check:
    require_link:
      min: 1
      node_type: requirement
      type: satisfies
  pack: design@1.0.0
---
**Why:** a part that satisfies no requirement is either missing its requirement or not needed.

**Fix:** add a `satisfies` link to a requirement, or write the missing requirement.
