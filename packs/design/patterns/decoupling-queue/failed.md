---
id: failed
type: queue
parent: queue
title: Failed messages
plane: dsn--design
fields:
  kind: queue
  delivery: at-least-once
---
Messages that could not be processed after the retries. Nothing is lost: someone looks at them, fixes the cause and puts them back, or discards them on purpose.
