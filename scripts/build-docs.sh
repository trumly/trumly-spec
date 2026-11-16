#!/usr/bin/env bash
# Builds the API reference (rustdoc) for every crate and copies it to DEST (replacing what is there).
#   scripts/build-docs.sh [DEST]      DEST defaults to target/api-docs
set -euo pipefail
cd "$(dirname "$0")/.."
DEST="${1:-target/api-docs}"
RUSTDOCFLAGS="--html-in-header docs/rustdoc-header.html -D warnings" cargo doc --no-deps --workspace --lib --bins
rm -rf "$DEST"
mkdir -p "$DEST"
cp -R target/doc/. "$DEST"/
# The generated embedded-packs file is built from absolute paths on this machine; do not publish its source view.
rm -rf "$DEST"/src/*/home "$DEST"/src/*/Users "$DEST"/src/*/tmp
echo "API reference -> $DEST ($(du -sh "$DEST" | cut -f1))"
printf '<!doctype html><meta charset="utf-8"><title>trumly spec reference</title><meta http-equiv="refresh" content="0; url=/reference"><a href="/reference">trumly spec reference</a>\n' > "$DEST/index.html"
