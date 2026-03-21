#!/bin/bash
# Compare source code and documentation metrics across landscape projects.
#
# Usage:
#   bash scripts/loc-landscape.sh              # all projects
#   bash scripts/loc-landscape.sh mengram      # single project
#   bash scripts/loc-landscape.sh --docs       # documentation only
#
# Requires: tokei (brew install tokei)
# Requires: sandbox/ clones (see book/src/landscape/process.md)
#
# To add a project: add a case to the source_metrics function below.

set -euo pipefail

SANDBOX="sandbox"

source_metrics() {
    local name=$1
    echo "=== ${name} (source) ==="
    case "$name" in
        kb-mcp)           tokei src/ tests/ ;;
        hipocampus)       tokei "${SANDBOX}/hipocampus/" ;;
        obsidian-web-mcp) tokei "${SANDBOX}/obsidian-web-mcp/" -t Python ;;
        mengram)          tokei "${SANDBOX}/mengram/" -t Python ;;
        hmem)             tokei "${SANDBOX}/hmem/" -t TypeScript ;;
        mnemex)           tokei "${SANDBOX}/mnemex/src/" ;;
        ori-mnemos)       tokei "${SANDBOX}/Ori-Mnemos/" -t TypeScript ;;
        *)                echo "  Unknown project: ${name}" ;;
    esac
    echo ""
}

docs_metrics() {
    local name=$1
    local path
    if [ "$name" = "kb-mcp" ]; then
        path="."
    else
        path="${SANDBOX}/${name}"
    fi

    echo "=== ${name} (docs) ==="
    if [ -d "$path" ]; then
        tokei "$path" -t Markdown
    else
        echo "  (not found — clone to ${SANDBOX}/)"
    fi
    echo ""
}

ALL_PROJECTS="kb-mcp hipocampus obsidian-web-mcp mengram hmem mnemex ori-mnemos"

docs_only=false
filter=""

for arg in "$@"; do
    case "$arg" in
        --docs) docs_only=true ;;
        *) filter="$arg" ;;
    esac
done

if [ -n "$filter" ]; then
    if [ "$docs_only" = true ]; then
        docs_metrics "$filter"
    else
        source_metrics "$filter"
        docs_metrics "$filter"
    fi
else
    if [ "$docs_only" = true ]; then
        echo "=== Documentation Metrics (Markdown) ==="
        echo ""
        for name in $ALL_PROJECTS; do docs_metrics "$name"; done
    else
        echo "=== Source Code Metrics ==="
        echo ""
        for name in $ALL_PROJECTS; do source_metrics "$name"; done
        echo "=== Documentation Metrics (Markdown) ==="
        echo ""
        for name in $ALL_PROJECTS; do docs_metrics "$name"; done
    fi
fi
