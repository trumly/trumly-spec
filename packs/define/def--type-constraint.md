---
id: def--type-constraint
type: type
title: constraint
fields:
  name: constraint
  plane: def--intent
  description: 'Use for a limit imposed from outside that the design must live within: a law, a budget,
    a deadline, a mandated technology. If it can be negotiated, it is a requirement instead.'
  shape: shield
  fields:
  - name: statement
    kind: text
    required: true
  - name: kind
    kind: choice
    required: true
    values:
    - legal
    - budget
    - schedule
    - technology
    - organisation
    - other
  - name: source
    kind: text
    label: Who or what imposes it
    required: true
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - dropped
  - name: accepted-by
    kind: text
    label: Accepted by (a person)
  - name: accepted-on
    kind: text
    label: Accepted on (date)
  pack: define@1.1.0
---
**Ask:** What is fixed that we did not choose? Who set it? What does it limit?

**Watch for:** preferences disguised as constraints. If it can be negotiated it is a requirement or a decision. Link it to what it limits with `governs`, including design nodes such as a cloud provider.
