---
id: entity-client
type: data-entity
title: Client
plane: dm--logical
links:
- type: data-contains
  to: attr-client-name
- type: data-has-many
  to: entity-booking
- type: data-realised-by
  to: table-client
---
