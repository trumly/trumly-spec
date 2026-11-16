---
id: constraint-slot-unique
type: data-constraint
title: One booking per slot per client
plane: dm--physical
fields:
  kind: unique
links:
- type: data-constrains
  to: table-booking
---
