---
id: gov--pack
type: pack
title: Governance
fields:
  name: governance
  namespace: gov
  version: 1.2.0
  description: 'Why the design is the way it is: decisions with the options weighed and what followed,
    and the principles choices are checked against. Rules ask for two options, exactly one chosen, a recorded
    consequence and a clean supersedes chain.'
  license: Apache-2.0
  publisher: trumly
  default-pattern: record-a-decision
  role: core
  use-when: Whenever a choice would be hard or costly to reverse, or someone would ask why it was made.
  requires:
  - links@^1.0.0
  - define@^1.0.0
  - design@^1.0.0
---
