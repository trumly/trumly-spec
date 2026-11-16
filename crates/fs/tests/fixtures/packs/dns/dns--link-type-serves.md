---
id: dns--link-type-serves
type: link-type
title: serves
fields:
  name: serves
  description: This hostname leads to a component, an interface or an environment.
  inverse: served-by
  from:
  - hostname
  to:
  - component
  - interface
  - environment
  pack: dns@0.1.0
---
