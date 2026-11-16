---
id: zone-public
type: network-zone
title: Public edge
plane: sdd--topology
fields:
  exposure: public
links:
- type: in-region
  to: region-london
- type: connects-to
  to: zone-private
---
