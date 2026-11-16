---
id: journey-order-ahead
type: journey
title: Sam orders a flat white from the street
plane: ux--flow
fields:
  outcome: Drink waiting when Sam walks in
links:
- type: has-step
  to: step-pick-drink
- type: has-step
  to: step-pay
- type: has-step
  to: step-collect
- type: satisfies
  to: req-order-ahead
- type: satisfies
  to: req-pay-online
---
