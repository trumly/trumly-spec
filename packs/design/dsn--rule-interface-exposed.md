---
id: dsn--rule-interface-exposed
type: rule
title: Interfaces have a provider
fields:
  name: Interfaces have a provider
  statement: Every interface is exposed by a component
  severity: error
  applies_to:
    type: interface
  check:
    require_link:
      dir: in
      min: 1
      node_type: component
      type: exposes
  pack: design@1.1.0
---
**Why:** an interface nobody offers cannot be called.

**Fix:** add an `exposes` link from the component that offers it.
