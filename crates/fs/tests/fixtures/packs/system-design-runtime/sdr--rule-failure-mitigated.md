---
id: sdr--rule-failure-mitigated
type: rule
title: Every failure mode has a mitigation
fields:
  statement: Every failure mode has a mitigation
  severity: error
  applies_to:
    type: failure-mode
    impact: outage
  check:
    require_link:
      type: mitigated-by
  pack: system-design-runtime@0.1.0
---
