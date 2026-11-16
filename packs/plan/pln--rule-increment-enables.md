---
id: pln--rule-increment-enables
type: rule
title: Increments enable something
fields:
  name: Increments enable something
  statement: Every increment enables at least one capability
  severity: warn
  applies_to:
    type: increment
  check:
    require_link:
      min: 1
      node_type: capability
      type: enables
  pack: plan@1.0.0
---
**Why:** an increment with no capability delivers no value anyone asked for.

**Fix:** add an `enables` link to the capability it unlocks.
