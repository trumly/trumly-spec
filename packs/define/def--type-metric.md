---
id: def--type-metric
type: type
title: metric
fields:
  name: metric
  plane: def--intent
  description: 'Use for a named, measurable quantity: a success measure for a goal, a target for a quality
    requirement, or an expected volume. Say how it is measured, its target, and for volumes the threshold
    that should trigger a rethink.'
  shape: chevron
  fields:
  - name: kind
    kind: choice
    required: true
    values:
    - success
    - quality
    - volume
  - name: unit
    kind: text
    required: true
  - name: direction
    kind: choice
    required: true
    values:
    - higher-is-better
    - lower-is-better
  - name: target
    kind: number
    required: true
  - name: threshold
    kind: number
    label: Threshold that triggers a rethink
  - name: current
    kind: number
    label: Current value
  - name: how-measured
    kind: text
    required: true
  - name: review-by
    kind: text
    label: Review by (date)
  pack: define@1.2.0
---
**Ask:** What number would tell you? Where does it come from? What value is good enough, and what value means the design needs to change?

One metric type serves three jobs, chosen by `kind`:
- **success**: how a goal is judged
- **quality**: the target behind a quality requirement
- **volume**: expected load or size, with a `threshold` that says when to revisit the design

The threshold is what lets a design react to growth later.
