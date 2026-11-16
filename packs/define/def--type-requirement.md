---
id: def--type-requirement
type: type
title: requirement
fields:
  name: requirement
  plane: def--requirements
  description: Use for one precise, testable statement of what the system must do (functional) or how
    well it must do it (quality). Each one traces up to a capability or goal and down to the proof that
    it is met.
  shape: document
  fields:
  - name: statement
    kind: text
    required: true
  - name: kind
    kind: choice
    required: true
    values:
    - functional
    - quality
  - name: quality-attribute
    kind: choice
    label: Quality attribute (quality only)
    values:
    - functional-suitability
    - performance-efficiency
    - compatibility
    - usability
    - reliability
    - security
    - maintainability
    - portability
  - name: pattern
    kind: choice
    values:
    - ubiquitous
    - event-driven
    - state-driven
    - unwanted-behaviour
    - optional-feature
    - complex
  - name: priority
    kind: choice
    required: true
    values:
    - must
    - should
    - could
  - name: status
    kind: choice
    required: true
    values:
    - draft
    - proposed
    - accepted
    - planned
    - built
    - verified
    - dropped
  - name: accepted-by
    kind: text
    label: Accepted by (a person)
  - name: accepted-on
    kind: text
    label: Accepted on (date)
  - name: rationale
    kind: text
  pack: define@1.1.0
---
**Ask:** What exactly must the system do or be? Can one test or measurement settle whether it is met? Is it one thing, not two?

**Pattern (EARS), optional but helpful:**
- ubiquitous: "The system shall…"
- event-driven: "When X, the system shall…"
- state-driven: "While X, the system shall…"
- unwanted behaviour: "If X, then the system shall…"
- optional feature: "Where X, the system shall…"

**Watch for:** vague words (fast, easy, secure). Quality requirements must point at a metric with a number. Functional requirements need an acceptance criterion.
