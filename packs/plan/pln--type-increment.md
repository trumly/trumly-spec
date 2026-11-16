---
id: pln--type-increment
type: type
title: increment
fields:
  name: increment
  plane: pln--plan
  description: Use for a releasable slice of value that enables one or more capabilities and groups the
    work packages needed for it.
  shape: chevron
  fields:
  - name: status
    kind: choice
    required: true
    values:
    - planned
    - in-progress
    - built
    - verified
    - dropped
  - name: target-date
    kind: text
  - name: body
    kind: text
    label: Description
    required: true
  pack: plan@1.0.0
---
**Ask:** What is the first slice that is useful on its own? What comes after?

Order increments with `follows`. A target date is optional and is a forecast, not a promise. The name `release` is taken by Helm, hence increment.

**Watch:** an increment that enables no capability, or one marked verified while its work is not.
