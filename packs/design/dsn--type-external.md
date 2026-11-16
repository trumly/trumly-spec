---
id: dsn--type-external
type: type
title: external
fields:
  name: external
  plane: dsn--design
  description: Use for a system or service outside this design that it depends on or that depends on it,
    such as a payment provider or the company's sign-in service.
  shape: cloud
  fields:
  - name: owner
    kind: text
    required: true
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.0.0
---
**Ask:** What do we rely on that we do not build or run? Who owns it?

Name an owner so there is someone to ask when it fails. An outside system that calls us is also an external, linked with `uses`.

**Watch:** an external with no owner or no plan for when it is down.
