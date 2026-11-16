---
id: sdr--type-failure-mode
type: type
title: failure-mode
fields:
  name: failure-mode
  plane: sdr--behaviour
  description: A way a component can fail.
  shape: octagon 2d
  fields:
  - name: likelihood
    kind: choice
    values:
    - rare
    - possible
    - likely
  - name: impact
    kind: choice
    values:
    - minor
    - degraded
    - outage
  pack: system-design-runtime@0.1.0
---
