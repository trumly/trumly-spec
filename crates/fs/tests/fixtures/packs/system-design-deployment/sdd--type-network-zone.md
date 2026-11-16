---
id: sdd--type-network-zone
type: type
title: network-zone
fields:
  name: network-zone
  plane: sdd--topology
  description: A part of the network with one level of exposure.
  shape: square 2d
  fields:
  - name: exposure
    kind: choice
    values:
    - public
    - private
    - restricted
    required: true
  pack: system-design-deployment@0.1.0
---
