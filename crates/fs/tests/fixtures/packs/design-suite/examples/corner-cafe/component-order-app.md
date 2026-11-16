---
id: component-order-app
type: component
title: Order app
plane: sd--design
fields:
  kind: app
  owner: Café team
links:
- type: runs-in
  to: environment-phones
- type: satisfies
  to: req-order-ahead
- type: satisfies
  to: req-pickup-time
- type: depends-on
  to: component-orders-service
---
