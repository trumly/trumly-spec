---
id: api--rule-built-served
type: rule
title: A built endpoint is served by something
fields:
  statement: A built endpoint is served by something
  severity: warn
  applies_to:
    type: api-endpoint
    status: built
  check:
    require_link:
      type: served-by
  pack: api@0.1.0
---
