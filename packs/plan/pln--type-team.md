---
id: pln--type-team
type: type
title: team
fields:
  name: team
  plane: pln--plan
  description: 'Use for a group that builds or reviews work: a human team or an AI agent. Every work package
    is owned by one.'
  shape: person
  fields:
  - name: kind
    kind: choice
    required: true
    values:
    - human
    - agent
  - name: focus
    kind: text
    required: true
  - name: body
    kind: text
    label: Description
    required: true
  pack: plan@1.0.0
---
**Ask:** Who builds this? Who reviews? Is the builder a person's team or an AI agent?

A team is a role, not a named individual. An AI agent is a team of kind `agent`; give it a focus narrow enough that its output can be reviewed.

**Watch:** work with no team, or an agent's work with no human reviewer.
