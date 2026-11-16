---
id: api--rule-endpoint-traces
type: rule
title: Every endpoint satisfies a requirement
fields:
  statement: Every endpoint satisfies a requirement
  severity: warn
  applies_to:
    type: api-endpoint
  check:
    require_link:
      type: satisfies
  pack: api@0.1.0
---

An endpoint nobody asked for is surface to defend for ever.
