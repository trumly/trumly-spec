---
id: sdr--link-type-mitigated-by
type: link-type
title: mitigated-by
fields:
  name: mitigated-by
  description: This failure is handled by.
  inverse: mitigates
  from:
  - failure-mode
  to:
  - component
  - queue
  - datastore
  pack: system-design-runtime@0.1.0
---
