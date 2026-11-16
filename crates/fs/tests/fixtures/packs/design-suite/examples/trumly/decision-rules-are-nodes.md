---
id: decision-rules-are-nodes
type: decision
title: Rules are nodes in the model
plane: adr--decisions
fields:
  status: accepted
  owner: Mat
  decided_on: 2026-10
links:
- type: governs
  to: component-core
- type: governs
  to: entity-rule
- type: considers
  to: option-rules-are-nodes-1
- type: considers
  to: option-rules-are-nodes-2
- type: has-consequence
  to: consequence-rules-are-nodes
---

Rules are versioned, reviewed and linked like everything else.
