---
id: def--type-goal
type: type
title: goal
fields:
  name: goal
  plane: def--intent
  description: 'Use for an outcome the system exists to achieve, stated as a result rather than a feature.
    Every goal needs a measure of success. Keep the list short: three to five goals is plenty.'
  shape: star
  fields:
  - name: statement
    kind: text
    required: true
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - achieved
    - dropped
  - name: accepted-by
    kind: text
    label: Accepted by (a person)
  - name: accepted-on
    kind: text
    label: Accepted on (date)
  pack: define@1.2.0
---
**Ask:** What would be different if this succeeds? How would you know, with a number? Who cares most?

**Watch for:** goals that describe features ("build a dashboard") rather than outcomes. A goal needs a `measured-by` metric before it can be accepted.

Capabilities link up to it with `derives-from`.
