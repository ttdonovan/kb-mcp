#!/bin/bash
# Shared entrypoint for all kb-mcp container agents.
#
# Copies mounted config.toml into ZeroClaw's writable config dir (~/.zeroclaw).
# The config dir is a tmpfs so we can't mount files into it directly —
# this script bridges read-only config mounts to the writable runtime dir.

set -euo pipefail

ZEROCLAW_DIR="${HOME}/.zeroclaw"
mkdir -p "${ZEROCLAW_DIR}/workspace"

# Copy config.toml from the read-only mount into the writable tmpfs
if [ -f /workspace/config/config.toml ]; then
    cp /workspace/config/config.toml "${ZEROCLAW_DIR}/config.toml"
    chmod 600 "${ZEROCLAW_DIR}/config.toml"
fi

# Symlink skills into the ZeroClaw workspace so they're discovered
if [ -d /workspace/skills ]; then
    ln -sf /workspace/skills "${ZEROCLAW_DIR}/workspace/skills"
fi

# Link ONNX model to where memvid-core expects it (hybrid search)
if [ -d /opt/memvid/text-models ]; then
    mkdir -p "${HOME}/.cache/memvid"
    ln -sf /opt/memvid/text-models "${HOME}/.cache/memvid/text-models"
fi

exec "$@"
