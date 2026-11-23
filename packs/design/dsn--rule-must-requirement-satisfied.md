---
id: dsn--rule-must-requirement-satisfied
type: rule
title: Musts are designed for
fields:
  name: Musts are designed for
  statement: Every accepted must-have requirement is satisfied by a design node
  severity: warn
  applies_to:
    priority: must
    status: accepted
    type: requirement
  check:
    require_link:
      dir: in
      min: 1
      type: satisfies
  pack: design@1.1.0
---
**Why:** an accepted must-have that nothing in the design satisfies is a promise nobody is keeping.

**Fix:** add a `satisfies` link from the component, interface or datastore that delivers it.
