---
id: component-fs
type: component
title: Folder reader
plane: sd--design
parent: group-engine
fields:
  kind: library
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-core
- type: reads
  to: datastore-model-folder
- type: runs-in
  to: environment-dev-machine
---
