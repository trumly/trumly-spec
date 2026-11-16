---
id: str--rule-bet-delivered
type: rule
title: An active bet gives rise to something
fields:
  statement: An active bet gives rise to something
  severity: info
  applies_to:
    type: bet
    status: active
  check:
    require_link:
      type: derives-from
      dir: in
  pack: strategy@0.1.0
---

A bet nobody turned into work is not being pursued.
