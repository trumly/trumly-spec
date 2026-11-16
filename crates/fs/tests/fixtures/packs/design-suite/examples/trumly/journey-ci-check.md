---
id: journey-ci-check
type: journey
title: CI fails a pull request that breaks the design rules
plane: ux--flow
fields:
  outcome: A broken design never merges
links:
- type: has-step
  to: step-push
- type: has-step
  to: step-read-findings
- type: satisfies
  to: requirement-validate
- type: satisfies
  to: requirement-conformance
---
