# kb-mcp — MCP server + CLI for markdown knowledge bases

# Default recipe — show available commands
default:
    @just --list

# =============================================================================
# Build
# =============================================================================

# Build (debug)
build:
    @cargo build

# Build (release)
release:
    @cargo build --release

# Check without building
check:
    @cargo check

# Lint with clippy
clippy:
    @cargo clippy

# Run tests
test:
    @cargo test

# Install to ~/.cargo/bin
install:
    @cargo install --path .

# =============================================================================
# Book
# =============================================================================

# Build mdBook documentation
book-build:
    @cd book && mdbook build

# Serve mdBook with live reload
book-serve:
    @cd book && mdbook serve --open

# =============================================================================
# Dev
# =============================================================================

# Run CLI (e.g., just run list-sections)
run *args:
    @cargo run -- {{args}}

# Search shorthand (e.g., just search "rate limits")
search query:
    @cargo run -- search --query "{{query}}"

# =============================================================================
# Agent
# =============================================================================

# Build researcher container image (BM25 only)
agent-build:
    @docker compose --profile dev build

# Build researcher container with hybrid search (BM25 + vector)
agent-build-hybrid:
    @docker compose --profile dev build --build-arg KB_MCP_FEATURES=hybrid

# Interactive research session
agent-research:
    @docker compose --profile dev run --rm researcher zeroclaw agent

# Research a specific topic (e.g., just agent-research-topic "HNSW vector search")
agent-research-topic topic:
    @docker compose --profile dev run --rm researcher zeroclaw agent -m "Research the following topic and add relevant findings to the vault: {{topic}}"

# Check what the agent can see in the vault
agent-vault-status:
    @docker compose --profile dev run --rm researcher kb-mcp list-sections

# =============================================================================
# Metrics (requires: brew install tokei)
# =============================================================================

# Line counts by language
loc:
    @tokei src/ tests/

# Largest files by code lines
loc-top:
    @tokei src/ tests/ --files --sort code

# Full project stats (includes docs, config, vault)
loc-all:
    @tokei . --exclude target --exclude book/book

# Compare against landscape projects (requires sandbox/ clones)
loc-landscape:
    @bash scripts/loc-landscape.sh

# Landscape docs metrics only
loc-landscape-docs:
    @bash scripts/loc-landscape.sh --docs

# Single project metrics (e.g., just loc-project mengram)
loc-project name:
    @bash scripts/loc-landscape.sh {{name}}
