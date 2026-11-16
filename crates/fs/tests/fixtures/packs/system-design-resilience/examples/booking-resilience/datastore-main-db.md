---
id: datastore-main-db
type: datastore
title: Main database
plane: sd--design
fields:
  kind: relational
  owner: Mat
links:
- type: runs-in
  to: env-production
- type: backed-up-by
  to: backup-main-db
- type: recovers-to
  to: rec-main-db
---
