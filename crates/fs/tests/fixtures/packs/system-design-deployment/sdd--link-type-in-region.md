---
id: sdd--link-type-in-region
type: link-type
title: in-region
fields:
  name: in-region
  description: A zone or environment lies in this region.
  inverse: contains
  from:
  - network-zone
  - environment
  to:
  - region
  pack: system-design-deployment@0.1.0
---
