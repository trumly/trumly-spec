---
id: abt--rule-overview-complete
type: rule
title: Overview is complete
fields:
  name: Overview is complete
  statement: The overview states purpose, audience, owner and stage
  severity: error
  applies_to:
    id: overview
  check:
    require_fields:
    - purpose
    - audience
    - owner
    - stage
  pack: about@1.0.0
---
**Why:** every reader, human or AI, starts here.

**Fix:** fill in purpose, audience, owner and stage on the node with id `overview`.
