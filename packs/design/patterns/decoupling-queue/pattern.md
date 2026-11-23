---
id: pattern
type: pattern
title: Decoupling queue
fields:
  title: Decoupling queue
  description: A queue between a producer and a worker so the producer never waits and bursts are absorbed, with a place for messages that keep failing. Rename the nodes to your work, scale the workers, and link each component to a requirement and an environment.
  use: starter
  requires:
  - design
  root: queue
  prefix: queue
---
