---
id: sdr--pack
type: pack
title: System design runtime
fields:
  name: system-design-runtime
  namespace: sdr
  version: 0.1.0
  description: 'Deep module: how the system behaves under load and failure. Service level objectives, the
    capacity it is sized for and how it can fail. Rules are errors: a service with an objective needs a
    way to know it is missed, and every failure mode needs a mitigation.'
  license: Apache-2.0
  publisher: trumly
  requires:
  - system-design@^0.3.0
---
