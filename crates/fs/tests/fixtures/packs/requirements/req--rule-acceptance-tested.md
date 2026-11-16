---
id: req--rule-acceptance-tested
type: rule
title: Every acceptance criterion has a test
fields:
  statement: Every acceptance criterion has a test
  severity: info
  applies_to:
    type: acceptance
  check:
    require_link:
      type: verified-by
      node_type: test-case
  pack: requirements@0.1.0
---
