---
id: dns--type-dns-record
type: type
title: dns-record
fields:
  name: dns-record
  plane: dns--naming
  description: A line in a domain's DNS zone that answers for a hostname.
  shape: document
  fields:
  - name: kind
    kind: choice
    values:
    - A
    - AAAA
    - CNAME
    - MX
    - TXT
    - NS
    - CAA
    - SRV
    required: true
  - name: value
    kind: text
  - name: ttl
    kind: number
  pack: dns@0.1.0
---
