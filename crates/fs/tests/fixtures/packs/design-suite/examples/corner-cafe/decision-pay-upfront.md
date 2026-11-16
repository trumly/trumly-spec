---
id: decision-pay-upfront
type: decision
title: Customers pay when they order, not at the counter
plane: adr--decisions
fields:
  status: accepted
  owner: Café team
  decided_on: 2026-10
links:
- type: considers
  to: option-pay-upfront-1
- type: considers
  to: option-pay-upfront-2
- type: has-consequence
  to: consequence-pay-upfront
- type: governs
  to: component-orders-service
---

Unpaid orders make the drink ready for nobody.
