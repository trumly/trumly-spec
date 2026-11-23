---
id: abt--type-overview
type: type
title: overview
fields:
  name: overview
  plane: abt--about
  description: 'Use once per model, to say what the model is for: the system in one sentence, who reads
    the design, who owns it, which stage it is in and where the other sources of truth live. Every reader
    and agent sees it first.'
  shape: circle 3d
  fields:
  - name: purpose
    kind: text
    label: What is being built, for whom
    required: true
  - name: audience
    kind: text
    label: Who reads this design
    required: true
  - name: owner
    kind: text
    label: Who owns it
    required: true
  - name: stage
    kind: choice
    required: true
    values:
    - define
    - design
    - plan
    - build
  - name: source-of-truth
    kind: text
    label: Where code, backlog and other sources live
  pack: about@1.1.0
---
**Ask:** In one sentence, what is being built and for whom? Who will read this design, and who owns it? Which stage is it in? Where do the code, backlog and other documents live?

There is exactly one overview, with the id `overview`. Agents read it first. The system's measurable outcomes belong in goals, not here, and the out-of-scope list is read from capabilities marked `out`.

Link it to the best entry points with `starts-at`.
