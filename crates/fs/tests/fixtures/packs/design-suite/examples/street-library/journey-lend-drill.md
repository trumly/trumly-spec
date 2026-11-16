---
id: journey-lend-drill
type: journey
title: Ali borrows Jo's drill for the weekend
plane: ux--flow
fields:
  outcome: The drill is back with Jo on Monday
links:
- type: has-step
  to: step-find
- type: has-step
  to: step-ask
- type: has-step
  to: step-hand-back
- type: satisfies
  to: req-list-item
- type: satisfies
  to: req-borrow
- type: satisfies
  to: req-return-record
---
