---
id: dns--rule-redirect-acyclic
type: rule
title: Hostnames do not redirect to each other in a circle
fields:
  statement: Hostnames do not redirect to each other in a circle
  severity: warn
  applies_to:
    type: hostname
  check:
    acyclic: redirects-to
  pack: dns@0.1.0
---
