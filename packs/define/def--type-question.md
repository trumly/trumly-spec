---
id: def--type-question
type: type
title: question
fields:
  name: question
  plane: def--intent
  description: Use for anything that needs a human decision before the design can move on. Agents raise
    a question instead of guessing. An open question blocks acceptance of what it is about.
  shape: diamond
  fields:
  - name: state
    kind: choice
    required: true
    values:
    - open
    - answered
  - name: raised-by
    kind: text
    required: true
  - name: answer
    kind: text
  - name: body
    kind: text
    label: Description
    required: true
  pack: define@1.1.0
---
**Ask (the AI asks itself):** Am I about to guess? If a decision is the person's to make, record it here instead of choosing.

Put the question in full in the description, since the title is only a short label. Link it to what it is about with `about`. While it is open, nothing it concerns can be accepted. When answered, record the answer and then update the affected nodes.
