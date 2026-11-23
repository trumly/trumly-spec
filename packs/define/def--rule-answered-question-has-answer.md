---
id: def--rule-answered-question-has-answer
type: rule
title: Answered needs an answer
fields:
  name: Answered needs an answer
  statement: An answered question records its answer
  severity: error
  applies_to:
    state: answered
    type: question
  check:
    require_fields:
    - answer
  pack: define@1.2.0
---
**Why:** the answer is the point. Without it the affected nodes cannot be updated.

**Fix:** fill in the answer, then update the nodes the question was about.
