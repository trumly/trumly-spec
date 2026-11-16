---
id: component-editor
type: component
title: Visual editor
plane: sd--design
parent: group-labs
fields:
  kind: app
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-wasm
- type: consumes
  to: interface-http-api
- type: runs-in
  to: environment-browser
- type: satisfies
  to: requirement-portable-names
- type: satisfies
  to: requirement-undo
- type: satisfies
  to: requirement-visual-edit
---

One self-contained page: the isometric map, tables, inspector and rule builder.
