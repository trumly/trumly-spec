---
id: req--rule-acceptance-has-requirement
type: rule
title: Every acceptance criterion belongs to a requirement
fields:
  statement: Every acceptance criterion belongs to a requirement
  severity: warn
  applies_to:
    type: acceptance
  check:
    require_link:
      type: verified-by
      dir: in
      node_type: requirement
  pack: requirements@0.1.0
---
