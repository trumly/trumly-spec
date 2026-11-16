---
id: component-cli
type: component
title: Command line
plane: sd--design
parent: group-tools
fields:
  kind: app
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-core
- type: depends-on
  to: component-fs
- type: exposes
  to: interface-cli
- type: reads
  to: datastore-model-folder
- type: runs-in
  to: environment-dev-machine
- type: runs-in
  to: environment-ci
- type: satisfies
  to: requirement-agent-overview
- type: satisfies
  to: requirement-validate
---
