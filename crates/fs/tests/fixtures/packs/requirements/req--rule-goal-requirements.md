---
id: req--rule-goal-requirements
type: rule
title: Every goal is served by a requirement
fields:
  statement: Every goal is served by a requirement
  severity: warn
  applies_to:
    type: goal
    status: active
  check:
    require_link:
      type: derives-from
      dir: in
      node_type: requirement
  pack: requirements@0.1.0
---
