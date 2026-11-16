---
id: sdz--rule-backup-tested
type: rule
title: Every backup has had a restore tested
fields:
  statement: Every backup has had a restore tested
  severity: warn
  applies_to:
    type: backup
  check:
    field_is:
      restore_tested:
        in:
        - true
  pack: system-design-resilience@0.1.0
---
