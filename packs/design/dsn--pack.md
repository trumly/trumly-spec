---
id: dsn--pack
type: pack
title: Design
fields:
  name: design
  namespace: dsn
  version: 1.0.0
  description: 'How the system is built, with no technology chosen: components, datastores, queues, interfaces,
    externals and the environments they run in. Enrichment packs (API detail, hosting, containers and
    the rest) attach to these nodes with details.'
  license: Apache-2.0
  publisher: trumly
  role: core
  use-when: 'When you are deciding how the system is put together: its parts, where data lives and what
    it talks to.'
  requires:
  - links@^1.0.0
  - define@^1.0.0
---
