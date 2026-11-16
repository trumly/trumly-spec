---
id: decision-flat-files
type: decision
title: Files stay flat; ids are <type>-<name>
plane: adr--decisions
fields:
  status: accepted
  owner: Mat
  decided_on: 2026-10
links:
- type: governs
  to: datastore-model-folder
- type: governs
  to: component-serve
- type: considers
  to: option-flat-files-1
- type: considers
  to: option-flat-files-2
- type: has-consequence
  to: consequence-flat-files
---

Directories carry no meaning, so nothing moves when a node changes plane, parent or type.
