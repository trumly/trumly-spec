---
id: acc-book-with-credit
type: acceptance
title: Booking spends one credit
plane: req--verification
fields:
  condition: A client with one credit books a free slot and has no credits left
links:
- type: verified-by
  to: test-book-session
---
