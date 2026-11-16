---
id: dns--rule-record-hostname
type: rule
title: Every DNS record answers for a hostname
fields:
  statement: Every DNS record answers for a hostname
  severity: warn
  applies_to:
    type: dns-record
  check:
    require_link:
      type: answers
      node_type: hostname
  pack: dns@0.1.0
---
