---
id: entity-item
type: data-entity
title: Item
plane: dm--logical
links:
- type: data-contains
  to: attribute-item-name
- type: data-contains
  to: attribute-item-owner
- type: data-has-many
  to: entity-loan
---
