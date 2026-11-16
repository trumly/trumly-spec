---
id: component-lending-app
type: component
title: Lending app
plane: sd--design
fields:
  kind: app
  owner: Street group
links:
- type: runs-in
  to: environment-phones
- type: satisfies
  to: req-list-item
- type: satisfies
  to: req-borrow
- type: satisfies
  to: req-return-record
- type: depends-on
  to: component-lending-service
---
