---
id: entity-order
type: data-entity
title: Order
plane: dm--logical
links:
- type: data-contains
  to: attribute-order-placed-at
- type: data-contains
  to: attribute-order-status
- type: data-has-many
  to: entity-drink
---
