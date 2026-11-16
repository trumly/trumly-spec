---
id: sdz--type-backup
type: type
title: backup
fields:
  name: backup
  plane: sdz--recovery
  description: A copy of data kept to restore from.
  shape: database up
  fields:
  - name: frequency
    kind: text
    required: true
  - name: retention
    kind: text
  - name: restore_tested
    kind: boolean
  pack: system-design-resilience@0.1.0
---
