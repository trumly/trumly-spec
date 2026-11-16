---
id: sd--type-interface
type: type
title: interface
fields:
  name: interface
  plane: sd--design
  description: 'A boundary other things talk to: an API, a file format, a command line.'
  shape: hexagon
  fields:
  - name: protocol
    kind: choice
    values:
    - http
    - grpc
    - graphql
    - event
    - file
    - cli
    - other
  pack: system-design@0.3.0
---
