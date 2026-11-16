---
id: component-core
type: component
title: Core engine (Rust)
plane: sd--design
parent: group-engine
fields:
  kind: library
  owner: trumly
  status: built
links:
- type: exposes
  to: interface-file-format
- type: runs-in
  to: environment-dev-machine
- type: runs-in
  to: environment-ci
- type: satisfies
  to: requirement-packs
- type: satisfies
  to: requirement-typed-graph
- type: satisfies
  to: requirement-validate
---

Pure: loads, validates and evaluates rules. No file or network access.
