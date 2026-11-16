---
id: rule-billing-owner
type: rule
title: Billing components have an owner
fields:
  statement: Billing components have an owner
  severity: warn
  applies_to:
    type: component
    under: billing
  check:
    require_fields:
    - owner
---
