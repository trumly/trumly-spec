---
id: type-service
type: type
fields:
  name: service
  fields:
    - { name: owner, kind: text, required: true }
    - { name: tier, kind: choice, values: [gold, silver], required: true }
    - { name: note, kind: text }
    - { name: flag, kind: boolean, required: "yes" }
---
