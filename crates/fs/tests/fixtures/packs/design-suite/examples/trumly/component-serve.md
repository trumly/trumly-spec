---
id: component-serve
type: component
title: Local server
plane: sd--design
parent: group-tools
fields:
  kind: service
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-core
- type: depends-on
  to: component-fs
- type: exposes
  to: interface-http-api
- type: reads
  to: datastore-model-folder
- type: writes
  to: datastore-model-folder
- type: writes
  to: datastore-view-state
- type: runs-in
  to: environment-dev-machine
- type: satisfies
  to: requirement-portable-names
- type: satisfies
  to: requirement-safe-concurrent
---

Serves the folder and the editor page on localhost, checks revisions before every write, and reports outside changes.
