---
id: dns--rule-production-named
type: rule
title: Every production environment is reached by a hostname
fields:
  statement: Every production environment is reached by a hostname
  severity: warn
  applies_to:
    type: environment
    kind: production
  check:
    require_link:
      type: serves
      dir: in
      node_type: hostname
  pack: dns@0.1.0
---
