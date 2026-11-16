---
id: bet-self-serve-booking
type: bet
title: Let clients book and pay for sessions themselves
plane: str--bets
fields:
  status: active
  confidence: medium
  owner: Mat
  horizon: Q1
links:
- type: targets
  to: segment-returning-riders
- type: addresses
  to: problem-booking-friction
- type: relies-on
  to: assumption-prefer-self-serve
- type: measured-by
  to: metric-self-serve-share
- type: aims-at
  to: icp-riding-schools
- type: addresses
  to: problem-phone-admin
---
