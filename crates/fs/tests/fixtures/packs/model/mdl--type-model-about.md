---
id: mdl--type-model-about
type: type
title: model-about
fields:
  name: model-about
  label: about
  description: 'One thing the model says about itself. The title is the statement (or the term, for a glossary entry), and the body (its description) is the detail, in markdown, and is required.'
  plane: mdl--about
  shape: circle 2d
  fields:
  - name: body
    kind: text
    required: true
    label: description
  - name: kind
    kind: choice
    required: true
    values:
    - purpose
    - audience
    - scope
    - status
    - owner
    - source-of-truth
    - convention
    - glossary
    - start-here
  pack: model@0.2.1
---
