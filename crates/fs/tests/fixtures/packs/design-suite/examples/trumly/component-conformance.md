---
id: component-conformance
type: component
title: Conformance suite (fixtures)
plane: sd--design
parent: group-spec
fields:
  kind: job
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-reference
- type: depends-on
  to: component-core
- type: runs-in
  to: environment-dev-machine
- type: runs-in
  to: environment-ci
- type: satisfies
  to: requirement-conformance
---

Inputs plus expected output. An implementation is correct when it matches every case.
