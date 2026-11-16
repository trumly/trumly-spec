#!/usr/bin/env bash
# Builds dist/trumly.wasm and proves it against the fixtures. Needs rustup and node.
set -euo pipefail
cd "$(dirname "$0")/.."
if ! command -v rustup >/dev/null; then
  echo "rustup not found. The wasm target needs rustup (Homebrew's 'rust' does not include it)."
  echo "Install: brew install rustup && rustup default stable   (or: curl https://sh.rustup.rs -sSf | sh)"
  exit 1
fi
rustup target add wasm32-unknown-unknown
cargo build -p trumly-wasm --release --target wasm32-unknown-unknown
mkdir -p dist
cp target/wasm32-unknown-unknown/release/trumly_wasm.wasm dist/trumly.wasm
ls -l dist/trumly.wasm
node crates/wasm/js/verify.mjs dist/trumly.wasm
node crates/wasm/js/verify-tools.mjs dist/trumly.wasm
