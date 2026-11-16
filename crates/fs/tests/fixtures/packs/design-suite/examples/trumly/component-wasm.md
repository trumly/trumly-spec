---
id: component-wasm
type: component
title: WebAssembly build of the core
plane: sd--design
parent: group-engine
fields:
  kind: library
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-core
- type: exposes
  to: interface-wasm-api
- type: runs-in
  to: environment-browser
---
