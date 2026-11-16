---
id: decision-phone-number
type: decision
title: People sign in with a phone number, not an account
plane: adr--decisions
fields:
  status: accepted
  owner: Street group
  decided_on: 2026-10
links:
- type: considers
  to: option-phone-number-1
- type: considers
  to: option-phone-number-2
- type: has-consequence
  to: consequence-phone-number
- type: governs
  to: component-lending-app
---

Neighbours will not make another password.
