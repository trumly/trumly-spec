---
id: dsn--rule-component-runs
type: rule
title: Components run somewhere
fields:
  name: Components run somewhere
  statement: Every component runs in an environment
  severity: warn
  applies_to:
    type: component
  check:
    require_link:
      min: 1
      node_type: environment
      type: runs-in
  pack: design@1.0.0
---
**Why:** a part that runs nowhere cannot be built or tested. Libraries and functions are the exception; they are shipped inside another component, so turn this off or ignore it for them.

**Fix:** add a `runs-in` link to an environment.
