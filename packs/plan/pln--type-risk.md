---
id: pln--type-risk
type: type
title: risk
fields:
  name: risk
  plane: pln--plan
  description: Use for something that could stop the build or the system working, with how likely and
    how bad it is and what will be done about it.
  shape: diamond
  fields:
  - name: likelihood
    kind: choice
    required: true
    values:
    - low
    - medium
    - high
  - name: impact
    kind: choice
    required: true
    values:
    - low
    - medium
    - high
  - name: mitigation
    kind: text
    required: true
  - name: state
    kind: choice
    required: true
    values:
    - open
    - mitigated
    - accepted
    - occurred
  - name: body
    kind: text
    label: Description
    required: true
  pack: plan@1.0.0
---
**Ask:** What could stop this being built or working? How likely, how bad, and what will we do about it?

Tie it to the assumption it comes from, so a failed assumption points at its risks. Every risk needs an owner and a mitigation.

**Watch:** a risk nobody owns, or one with no mitigation.
