---
id: api--rule-auth-scheme
type: rule
title: Every auth names its scheme
fields:
  statement: Every auth names its scheme
  severity: error
  applies_to:
    type: api-auth
  check:
    require_fields:
    - scheme
  pack: api@0.1.0
---
