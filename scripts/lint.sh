#!/usr/bin/env bash
# The one lint command: CI and release.sh run this. Format is checked only with --format (see rustfmt.toml).
set -euo pipefail
cd "$(dirname "$0")/.."
# collapsible_if: nested ifs read better than let-chains here; the other two are about shapes we use on purpose
cargo clippy --workspace --all-targets -- -D warnings -A clippy::collapsible_if -A clippy::type_complexity -A clippy::result_large_err
[ "${1:-}" = "--format" ] && cargo fmt --all -- --check
if command -v shellcheck >/dev/null; then shellcheck scripts/*.sh; else echo "shellcheck not installed: skipped" >&2; fi
