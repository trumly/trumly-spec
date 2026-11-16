---
id: def--type-concept
type: type
title: concept
fields:
  name: concept
  plane: def--capabilities
  description: Use for a term of the business or domain that the design relies on, such as an order, a
    job or a customer. It is the shared glossary, and the seed of the data model.
  shape: document
  fields:
  - name: definition
    kind: text
    required: true
  pack: define@1.1.0
---
**Ask:** Which words does the team use that an outsider would misread? What do they mean here?

Concepts later become entities in the data model. Keep the definition to a sentence or two, and link the capabilities and requirements that use the term with `refers-to`.
