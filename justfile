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
