---
id: dns--rule-domain-provider
type: rule
title: Every domain names the provider it is registered with
fields:
  statement: Every domain names the provider it is registered with
  severity: info
  applies_to:
    type: domain
  check:
    require_link:
      type: provided-by
      node_type: provider
  pack: dns@0.1.0
---
