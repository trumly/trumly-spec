---
id: dsn--type-interface
type: type
title: interface
fields:
  name: interface
  plane: dsn--design
  description: 'Use for a boundary other things talk to: an API, an event feed, a file format or a command
    line.'
  shape: hexagon
  fields:
  - name: protocol
    kind: choice
    required: true
    values:
    - http
    - grpc
    - graphql
    - event
    - file
    - cli
    - other
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.0.0
---
**Ask:** What do other things talk to? Who calls it, and in what style?

This is only the boundary: its name, who offers it and the protocol family. Endpoints, payloads and auth go in the API enrichment stack, linked with `details`.

**Watch:** an interface nobody exposes, or two components talking with no interface between them.
