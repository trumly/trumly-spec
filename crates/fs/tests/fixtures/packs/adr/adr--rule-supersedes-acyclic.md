---
id: adr--rule-supersedes-acyclic
type: rule
title: Supersedes never loops
fields:
  statement: Supersedes never loops
  severity: error
  applies_to:
    type: decision
  check:
    acyclic: supersedes
  pack: adr@0.1.0
---
