---
id: sdz--type-recovery-objective
type: type
title: recovery-objective
fields:
  name: recovery-objective
  plane: sdz--recovery
  description: How much data and time we can afford to lose.
  shape: shield 2d
  fields:
  - name: rto
    kind: text
    required: true
  - name: rpo
    kind: text
    required: true
  pack: system-design-resilience@0.1.0
---
