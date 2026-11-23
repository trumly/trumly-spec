---
id: def--pack
type: pack
title: Define
fields:
  name: define
  namespace: def
  version: 1.2.0
  description: 'Why the system exists and what it must do: goals and how success is measured, constraints,
    assumptions and open questions, then the actors, capabilities and glossary, and the precise requirements
    with the criteria that prove them. Rules keep every requirement traceable up to a goal and down to
    a test.'
  license: Apache-2.0
  publisher: trumly
  default-pattern: goal-to-proof
  role: core
  use-when: Always, before any design. Use it to capture what the people who asked for the system want,
    in their words, before choosing how to build it.
  requires:
  - links@^1.0.0
---
