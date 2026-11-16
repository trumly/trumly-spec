---
id: ux--rule-journey-persona
type: rule
title: Every journey is taken by a persona
fields:
  statement: Every journey is taken by a persona
  severity: error
  applies_to:
    type: journey
  check:
    require_link:
      type: takes
      dir: in
      node_type: persona
  pack: ux@0.1.0
---
