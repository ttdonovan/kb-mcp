#!/usr/bin/env bash
# Builds kb-mcp from this checkout (debug) and runs it, so /mcp reconnect picks up code changes.
# Nothing may reach stdout before the exec: it carries the MCP protocol.
set -euo pipefail

# The checkout: the plugin's own repo when loaded in place (`just claude`), else the session's project.
root=""
for dir in "${CLAUDE_PLUGIN_ROOT:-}/../.." "${CLAUDE_PROJECT_DIR:-}" "$PWD"; do
    if [[ -n "$dir" && -f "$dir/crates/kb-mcp-server/Cargo.toml" ]]; then
        root="$(cd "$dir" && pwd)"
        break
    fi
done
if [[ -z "$root" ]]; then
    echo "kb-dev: not in a kb-mcp checkout; start Claude Code with \`just claude\` from one" >&2
    exit 1
fi

# From the root, so rust-toolchain.toml applies.
cd "$root"
if ! cargo build --locked -p kb-mcp-server >&2; then
    echo "kb-dev: build failed; fix it, then reconnect kb-dev in /mcp" >&2
    exit 1
fi

# Your collections.ron if you have one (gitignored), else the example, which indexes this repo.
if [[ -z "${KB_MCP_CONFIG:-}" ]]; then
    KB_MCP_CONFIG="$root/collections.ron"
    [[ -f "$KB_MCP_CONFIG" ]] || KB_MCP_CONFIG="$root/collections.example.ron"
fi
export KB_MCP_CONFIG
# Keep the debug build's index apart from an installed kb-mcp's.
export KB_MCP_CACHE_DIR="${KB_MCP_CACHE_DIR:-$root/target/kb-mcp-dev/index}"

exec "${CARGO_TARGET_DIR:-$root/target}/debug/kb-mcp"
