---
id: dsn--type-component
type: type
title: component
fields:
  name: component
  plane: dsn--design
  description: 'Use for a part of the system that does work and that someone owns: a service, an app,
    a scheduled job, a library or a function. Every component should satisfy a requirement.'
  shape: square
  fields:
  - name: kind
    kind: choice
    required: true
    values:
    - service
    - app
    - job
    - library
    - function
  - name: owner
    kind: text
    required: true
  - name: status
    kind: choice
    required: true
    values:
    - proposed
    - planned
    - built
  - name: body
    kind: text
    label: Description
    required: true
  pack: design@1.1.0
---
**Ask:** What are the separate parts that do the work? Who looks after each one?

Split by what changes together and who owns it, not by technology. Use `parent` to nest a part inside a bigger one when you need more detail. A component is not a Docker container: that is an `image` run as a `container`. Keep the title to a short label and say the rest in the description.

**Watch:** a component with no owner, no requirement it satisfies, or nowhere it runs.
