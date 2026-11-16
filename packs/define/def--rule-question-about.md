---
id: def--rule-question-about
type: rule
title: Questions are about a node
fields:
  name: Questions are about a node
  statement: Every question is about a node
  severity: error
  applies_to:
    type: question
  check:
    require_link:
      min: 1
      type: about
  pack: define@1.1.0
---
**Why:** the question blocks what it is about.

**Fix:** add an `about` link.
