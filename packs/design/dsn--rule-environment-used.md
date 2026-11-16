---
id: dsn--rule-environment-used
type: rule
title: Environments host something
fields:
  name: Environments host something
  statement: Every environment hosts something
  severity: warn
  applies_to:
    type: environment
  check:
    require_link:
      dir: in
      min: 1
      type: runs-in
  pack: design@1.0.0
---
**Why:** an environment nothing runs in is an unused place.

**Fix:** add `runs-in` from the components, datastores or queues that run there.
