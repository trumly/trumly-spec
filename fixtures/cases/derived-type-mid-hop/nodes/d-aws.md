---
id: d-aws
type: derived
fields:
  name: aws-name
  template: '{env}-{region}-{details/@type/^stands-for/value}'
  applies_to: {type: rds}
---
