---
id: sd--rule-depends-acyclic
type: rule
title: Components do not depend on each other in a circle
fields:
  statement: Components do not depend on each other in a circle
  severity: warn
  applies_to:
    type: component
  check:
    acyclic: depends-on
  pack: system-design@0.3.0
---
