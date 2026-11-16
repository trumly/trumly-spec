---
id: adr--type-decision-option
type: type
title: decision-option
fields:
  name: decision-option
  plane: adr--decisions
  description: One way the decision could have gone.
  shape: hexagon 2d
  fields:
  - name: chosen
    kind: boolean
  - name: summary
    kind: text
  pack: adr@0.1.0
---
