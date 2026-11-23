---
id: gov--rule-option-belongs
type: rule
title: Options belong to a decision
fields:
  name: Options belong to a decision
  statement: Every decision option is considered by a decision
  severity: warn
  applies_to:
    type: decision-option
  check:
    require_link:
      dir: in
      min: 1
      node_type: decision
      type: considers
  pack: governance@1.2.0
---
**Why:** an option nobody considered is noise.

**Fix:** add `considers` from the decision it belongs to, or delete the option.
