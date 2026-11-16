---
id: sdr--type-slo
type: type
title: SLO
fields:
  name: slo
  plane: sdr--behaviour
  description: A promise about how well a component performs.
  shape: shield
  fields:
  - name: objective
    kind: text
    required: true
  - name: window
    kind: text
    required: true
  - name: alert
    kind: text
  pack: system-design-runtime@0.1.0
---
