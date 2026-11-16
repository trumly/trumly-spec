---
id: rule-comp-runs
type: rule
title: Every component runs in an environment
fields:
  statement: Every component runs in an environment
  severity: error
  applies_to:
    type: component
  check:
    require_link:
      type: runs-in
      to_plane: run
---
