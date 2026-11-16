---
id: dns--rule-hostname-record
type: rule
title: Every hostname has a record that answers for it
fields:
  statement: Every hostname has a record that answers for it
  severity: warn
  applies_to:
    type: hostname
  check:
    require_link:
      type: answers
      dir: in
      node_type: dns-record
  pack: dns@0.1.0
---
