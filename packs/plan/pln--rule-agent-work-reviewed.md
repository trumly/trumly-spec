---
id: pln--rule-agent-work-reviewed
type: rule
title: Agent work has a human
fields:
  name: Agent work has a human
  statement: Every work package is owned or reviewed by a human team
  severity: error
  applies_to:
    type: work-package
  check:
    any_of:
    - require_link:
        node_fields:
          kind: human
        node_type: team
        type: owned-by
    - require_link:
        node_fields:
          kind: human
        node_type: team
        type: reviewed-by
  pack: plan@1.0.0
---
**Why:** agents propose, humans accept. Someone human must own the work or review it.

**Fix:** give the work package a human team as owner, or add `reviewed-by` pointing at a human team.
