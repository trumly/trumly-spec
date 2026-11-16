---
id: rule-req-built
type: rule
title: Every requirement is implemented by a component
fields:
  statement: Every requirement is implemented by a component
  severity: warn
  applies_to:
    type: requirement
  check:
    require_link:
      type: implemented-by
      to_type: component
      to_plane: build
---
