---
id: biz--rule-positioning-rests
type: rule
title: Every positioning claim rests on a differentiator
fields:
  statement: Every positioning claim rests on a differentiator
  severity: warn
  applies_to:
    type: positioning
  check:
    require_link:
      type: rests-on
      node_type: differentiator
  pack: business@0.1.0
---
