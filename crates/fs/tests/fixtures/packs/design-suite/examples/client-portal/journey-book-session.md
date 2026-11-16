---
id: journey-book-session
type: journey
title: Book a session
plane: ux--flow
fields:
  outcome: A confirmed slot
links:
- type: has-step
  to: step-pick-slot
- type: has-step
  to: step-confirm
- type: satisfies
  to: req-book-session
---
