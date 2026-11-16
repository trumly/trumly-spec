---
id: biz--rule-plan-icp
type: rule
title: Every live pricing plan is priced for an ideal customer
fields:
  statement: Every live pricing plan is priced for an ideal customer
  severity: warn
  applies_to:
    type: pricing-plan
    status: live
  check:
    require_link:
      type: priced-for
      node_type: icp
  pack: business@0.1.0
---
