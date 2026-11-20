#!/usr/bin/env bash
# The lint gate shared by `just ci` and CI: fmt, clippy (BM25 and hybrid), tests, shellcheck, advisories, dependency age.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --locked --features kb-cli/hybrid,kb-mcp-server/hybrid -- -D warnings
cargo test --workspace --locked
shellcheck scripts/ci/*.sh plugins/*/scripts/*.sh
cargo audit
scripts/ci/crate-age.sh
