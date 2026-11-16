---
id: dns--rule-hostname-domain
type: rule
title: Every hostname is under a domain
fields:
  statement: Every hostname is under a domain
  severity: warn
  applies_to:
    type: hostname
  check:
    require_link:
      type: under
      node_type: domain
  pack: dns@0.1.0
---
