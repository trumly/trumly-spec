---
id: req--rule-must-accepted
type: rule
title: Every must-have requirement has an acceptance criterion
fields:
  statement: Every must-have requirement has an acceptance criterion
  severity: warn
  applies_to:
    type: requirement
    priority: must
  check:
    require_link:
      type: verified-by
      node_type: acceptance
  pack: requirements@0.1.0
---
