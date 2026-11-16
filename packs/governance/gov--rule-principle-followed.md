---
id: gov--rule-principle-followed
type: rule
title: Principles are applied
fields:
  name: Principles are applied
  statement: Every principle is followed or departed from by a decision
  severity: info
  applies_to:
    type: principle
  check:
    any_of:
    - require_link:
        dir: in
        type: complies-with
    - require_link:
        dir: in
        type: departs-from
  pack: governance@1.1.0
---
Advisory. A principle no decision follows or departs from may be unused or too vague to test.

**Fix:** link a decision with `complies-with` or `departs-from`.
