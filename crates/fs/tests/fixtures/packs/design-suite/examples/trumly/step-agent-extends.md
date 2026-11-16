---
id: step-agent-extends
type: step
title: Ask the agent to fill in the detail
plane: ux--flow
fields:
  kind: action
links:
- type: leads-to
  to: step-review
- type: happens-on
  to: screen-agent-chat
---
