---
id: def--type-actor
type: type
title: actor
fields:
  name: actor
  plane: def--capabilities
  description: Use for a kind of person (a role) who uses or runs the system. Every capability is performed
    by an actor.
  shape: person
  fields: []
  pack: define@1.2.0
---
**Ask:** Who uses it? Who runs it?

An actor is a role, not a named individual, and not the model's audience. An outside system that calls this one is an `external`, not an actor.
