---
id: abt--type-convention
type: type
title: convention
fields:
  name: convention
  plane: abt--about
  description: 'Use for a standing agreement about how this model is written: naming, level of detail,
    what not to model. Attach it to the type or plane it governs so an agent about to create one finds
    it.'
  shape: document up
  fields:
  - name: body
    kind: text
    label: Detail
    required: true
  pack: about@1.1.0
---
**Ask:** What rule of thumb would you want a newcomer, or an AI, to follow without being told each time?

One convention per node, so each can be found and linked. Use `describes` to attach it to the type or plane it governs.
