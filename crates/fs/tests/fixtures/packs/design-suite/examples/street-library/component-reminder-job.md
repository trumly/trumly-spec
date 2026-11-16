---
id: component-reminder-job
type: component
title: Reminder job
plane: sd--design
fields:
  kind: job
  owner: Street group
links:
- type: satisfies
  to: req-reminder
- type: depends-on
  to: external-sms
- type: runs-in
  to: environment-cloud
---
