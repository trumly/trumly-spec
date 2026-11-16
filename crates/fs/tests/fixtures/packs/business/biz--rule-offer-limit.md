---
id: biz--rule-offer-limit
type: rule
title: Every offer ends or is limited
fields:
  statement: Every offer ends or is limited
  severity: warn
  applies_to:
    type: offer
  check:
    any_of:
    - require_fields:
      - ends
    - require_fields:
      - limit
  pack: business@0.1.0
---

A deal with no end date or cap is just the price.
