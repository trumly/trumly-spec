---
id: api--type-api-schema
type: type
title: api-schema
fields:
  name: api-schema
  plane: api--contract
  description: The shape of a request, response or message body.
  shape: document
  label: schema
  fields:
  - name: format
    kind: choice
    values:
    - json
    - protobuf
    - avro
    - xml
    - form
  pack: api@0.1.0
---
