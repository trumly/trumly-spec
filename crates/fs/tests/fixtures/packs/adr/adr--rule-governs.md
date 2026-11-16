---
id: adr--rule-governs
type: rule
title: An accepted decision governs something
fields:
  statement: An accepted decision governs something
  severity: warn
  applies_to:
    type: decision
    status: accepted
  check:
    require_link:
      type: governs
  pack: adr@0.1.0
---

A decision that governs nothing cannot be followed or broken.
