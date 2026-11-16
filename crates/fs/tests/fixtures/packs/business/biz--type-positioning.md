---
id: biz--type-positioning
type: type
title: positioning
fields:
  name: positioning
  plane: biz--position
  description: The claim you make about where you stand. The title is the claim.
  shape: cloud
  fields:
  - name: status
    kind: choice
    values:
    - guess
    - evidenced
  - name: evidence
    kind: text
  pack: business@0.1.0
---
