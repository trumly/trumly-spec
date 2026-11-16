---
id: biz--rule-cost-weighed
type: rule
title: Every cost driver is weighed by an assumption
fields:
  statement: Every cost driver is weighed by an assumption
  severity: info
  applies_to:
    type: cost-driver
  check:
    require_link:
      type: drives
      dir: in
      node_type: unit-assumption
  pack: business@0.1.0
---
