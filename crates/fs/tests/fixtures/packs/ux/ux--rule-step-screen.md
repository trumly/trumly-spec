---
id: ux--rule-step-screen
type: rule
title: Every action step happens on a screen
fields:
  statement: Every action step happens on a screen
  severity: warn
  applies_to:
    type: step
    kind: action
  check:
    require_link:
      type: happens-on
      node_type: screen
  pack: ux@0.1.0
---
