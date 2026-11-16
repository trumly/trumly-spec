---
id: def--rule-criterion-verifies-requirement
type: rule
title: Criteria prove a requirement
fields:
  name: Criteria prove a requirement
  statement: Every acceptance criterion verifies a requirement
  severity: error
  applies_to:
    type: acceptance-criterion
  check:
    require_link:
      dir: in
      min: 1
      node_type: requirement
      type: verified-by
  pack: define@1.1.0
---
**Why:** an orphan test proves nothing anyone asked for.

**Fix:** link a requirement to it with `verified-by`.
