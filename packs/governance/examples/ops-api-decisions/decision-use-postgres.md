---
id: decision-use-postgres
type: decision
title: Database choice
plane: gov--decisions
fields:
  status: accepted
  accepted-by: Mat
  accepted-on: '2026-11-03'
  review-by: '2027-05-01'
links:
- type: considers
  to: decision-option-managed-postgres
- type: considers
  to: decision-option-document-store
- type: has-consequence
  to: consequence-one-engine
- type: governs
  to: datastore-ops-db
- type: derives-from
  to: requirement-status-freshness
- type: complies-with
  to: principle-prefer-managed
- type: relies-on
  to: assumption-job-volume
---
Use a managed relational database. Job data is highly structured and the reports need joins.
