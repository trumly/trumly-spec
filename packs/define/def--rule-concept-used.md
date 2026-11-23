---
id: def--rule-concept-used
type: rule
title: Glossary terms are used
fields:
  name: Glossary terms are used
  statement: Every concept is referred to by something
  severity: info
  applies_to:
    type: concept
  check:
    require_link:
      dir: in
      min: 1
      type: refers-to
  pack: define@1.2.0
---
Advisory. A glossary term that nothing refers to may be unnecessary, or the nodes that use it may be missing links.

**Fix:** add `refers-to` from the node that uses the term, or drop it.
