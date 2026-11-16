# Security

Please do not report a security problem in a public issue.

## Reporting

Use [private vulnerability reporting](https://github.com/trumly/trumly-spec/security/advisories/new) on this repository, or write to hello@trumly.co. Please include what you found, the version (`trumly --version`) and the steps to reproduce it. A short description is enough to start with.

We aim to acknowledge a report within 5 working days, and to tell you what we plan to do about it soon after. We will credit you in the fix unless you would rather we did not.

## What this covers

This repository: the specification, the reference core, the `trumly` command line, the `trumly-mcp` server and the core packs. Problems in the trumly editor or trumly cloud can go to the same address.

## Supported versions

trumly is at version 0.1, before 1.0. Fixes go into the latest release only.

## Dependencies

We pin dependencies in `Cargo.lock` and run `cargo audit` on every change.
