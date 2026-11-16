---
id: decision-yaml-1-2
type: decision
title: Frontmatter is YAML 1.2 core schema
plane: adr--decisions
fields:
  status: accepted
  owner: Mat
  decided_on: 2026-10
links:
- type: governs
  to: component-core
- type: governs
  to: component-reference
- type: considers
  to: option-yaml-1-2-1
- type: considers
  to: option-yaml-1-2-2
- type: has-consequence
  to: consequence-yaml-1-2
---

YAML 1.1 turns no/on/yes and 0755 into surprises; 1.2 does not.
