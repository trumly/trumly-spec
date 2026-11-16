---
id: biz--rule-icp-pain
type: rule
title: Every ideal customer is chosen from a segment that has a problem
fields:
  statement: Every ideal customer is chosen from a segment that has a problem
  severity: warn
  applies_to:
    type: icp
  check:
    path:
    - type: derives-from
      node_type: segment
    - type: suffers
      node_type: problem
  pack: business@0.1.0
---

If nobody hurts, nobody buys. Follow the customer to the segment it came from, then to the problem that segment has.
