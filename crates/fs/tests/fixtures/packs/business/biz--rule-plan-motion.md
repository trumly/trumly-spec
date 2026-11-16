---
id: biz--rule-plan-motion
type: rule
title: Every live pricing plan is sold through a motion
fields:
  statement: Every live pricing plan is sold through a motion
  severity: warn
  applies_to:
    type: pricing-plan
    status: live
  check:
    require_link:
      type: sold-by
      node_type: motion
  pack: business@0.1.0
---
