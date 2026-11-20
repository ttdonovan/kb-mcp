# kb-mcp

MCP server + CLI for markdown knowledge bases.

[Read the Docs](https://ttdonovan.github.io/kb-mcp/) ([for agents: llms.txt](https://ttdonovan.github.io/kb-mcp/llms.txt)) | [Why kb-mcp](https://ttdonovan.github.io/kb-mcp/product/) | [GitHub](https://github.com/ttdonovan/kb-mcp)

Indexes markdown collections into a searchable BM25 index and exposes them
as [MCP](https://modelcontextprotocol.io/) tools. Configured entirely via
RON files — no hardcoded paths or project-specific values.

## Quick Start

```sh
# Clone and build
git clone https://github.com/ttdonovan/kb-mcp.git
cd kb-mcp
just install  # installs both `kb` (CLI) and `kb-mcp` (MCP server)

# Try it — the example config indexes this project's own vault and docs
cp collections.example.ron collections.ron

# Search the included AI agent memory vault
kb list-sections
kb search --query "cognitive memory"
kb context --path "concepts/cognitive-memory-model.md"
kb get-document --path "concepts/retrieval-strategies.md"
```

To use with your own markdown collections, edit `collections.ron` to
point at your directories. See [Configuration](#configuration) below.

## Install

Prebuilt `kb` and `kb-mcp` binaries (BM25 only) for Linux (x86_64, arm64,
static musl) and macOS (arm64, x86_64) are attached to each
[GitHub Release](https://github.com/ttdonovan/kb-mcp/releases), with a
`SHA256SUMS` file and build provenance:

```sh
TAG=vX.Y.Z TARGET=aarch64-apple-darwin
curl -fsSLO "https://github.com/ttdonovan/kb-mcp/releases/download/$TAG/kb-mcp-$TAG-$TARGET.tar.gz"
gh attestation verify "kb-mcp-$TAG-$TARGET.tar.gz" --repo ttdonovan/kb-mcp
tar -xzf "kb-mcp-$TAG-$TARGET.tar.gz"   # then put kb and kb-mcp on your PATH
```

Or build a release from its tag (kb-mcp is not on crates.io):

```sh
cargo install --locked --git https://github.com/ttdonovan/kb-mcp --tag vX.Y.Z kb-cli kb-mcp-server
```

From a clone:

```sh
# Install both binaries (CLI `kb` + MCP server `kb-mcp`)
just install

# Or install individually
cargo install --path crates/kb-cli          # installs `kb`
cargo install --path crates/kb-mcp-server   # installs `kb-mcp`

# With hybrid search (BM25 + vector, requires ONNX model — see below)
cargo install --path crates/kb-cli --features hybrid
cargo install --path crates/kb-mcp-server --features hybrid
```

### Hybrid Search (optional)

Enable the `hybrid` feature for semantic vector search alongside BM25.
Conceptual queries like "how do agents share state?" will match documents
titled "shared memory" that keyword search alone would miss.

Requires the BGE-small-en-v1.5 ONNX model (~133MB):

```sh
# macOS: ~/Library/Caches/memvid/text-models/
# Linux: ~/.cache/memvid/text-models/
CACHE_DIR="${HOME}/Library/Caches/memvid/text-models"  # adjust for Linux
mkdir -p "$CACHE_DIR"
curl -L -o "$CACHE_DIR/bge-small-en-v1.5.onnx" \
  https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model.onnx
curl -L -o "$CACHE_DIR/bge-small-en-v1.5_tokenizer.json" \
  https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/tokenizer.json
```

When hybrid is enabled, search automatically uses RRF fusion of BM25 +
vector results. No query changes needed — agents get better results
transparently.

## Configuration

Create a `collections.ron` in your project root (or `~/.config/kb-mcp/collections.ron`):

```ron
(
    collections: [
        (
            name: "docs",
            path: "docs",
            description: "Project documentation",
            writable: false,
            sections: [
                (prefix: "guides", description: "How-to guides"),
                (prefix: "reference", description: "API reference"),
            ],
        ),
        (
            name: "notes",
            path: "notes",
            description: "Working notes",
            writable: true,
            sections: [],
        ),
    ],
)
```

Collection paths and `cache_dir` resolve relative to the config file's location. The `KB_MCP_CACHE_DIR` environment variable overrides `cache_dir`.

### Config resolution order

1. `--config <path>` CLI flag
2. `KB_MCP_CONFIG` environment variable
3. `./collections.ron` (current directory)
4. `~/.config/kb-mcp/collections.ron`

## MCP Server

Register in your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "kb": {
      "command": "kb-mcp",
      "args": []
    }
  }
}
```

For cross-project use, set `KB_MCP_CONFIG` to point to the config:

```json
{
  "mcpServers": {
    "kb": {
      "command": "kb-mcp",
      "env": {
        "KB_MCP_CONFIG": "/path/to/collections.ron"
      },
      "args": []
    }
  }
}
```

### Tools

| Tool | Description |
|------|-------------|
| `list_sections` | List collections with section doc counts and descriptions |
| `search` | Full-text search (BM25, or hybrid BM25+vector with `--features hybrid`) |
| `get_document` | Retrieve full document content by path or title |
| `kb_context` | Token-efficient briefing (frontmatter + summary only) |
| `kb_write` | Create a note in a writable collection |
| `reindex` | Rebuild the search index from disk |

## CLI

The `kb` binary provides CLI access to all tools:

```sh
kb list-sections
kb search --query "rate limits"
kb search --query "bevy" --collection skills
kb get-document --path "concepts/mcp-server-pattern.md"
kb context --path "concepts/mcp-server-pattern.md"
kb write --collection notes --title "My Note" --body "Content here"
kb reindex
```

## Documentation

There are two [mdBook](https://rust-lang.github.io/mdBook/) books, both built with [mdbook-llms](https://github.com/ttdonovan/mdbook-llms) so agents can read them as `llms.txt` and plain Markdown:

- [The technical book](https://ttdonovan.github.io/kb-mcp/), in `docs/src/`: install, configure, use, every tool and config field, and how to work on kb-mcp. Agents start at its [`llms.txt`](https://ttdonovan.github.io/kb-mcp/llms.txt).
- [The product book](https://ttdonovan.github.io/kb-mcp/product/), in `book/src/`: why kb-mcp exists, its goals and roadmap, and how it compares with other projects.

```sh
just docs setup   # install mdBook, mdbook-mermaid and mdbook-llms, once
just docs         # build the technical book into target/docs/html/
just docs serve   # preview it with live reload
just docs check   # fail on a warning, an empty chapter or a broken link
just book         # build the product book into target/book/html/ (also: serve, check)
```

## Prerequisites

```sh
cargo install mdbook
cargo install mdbook-mermaid
```

### Build and serve

```sh
just book-build   # Build to book/book/
just book-serve   # Serve with live reload at http://localhost:3000
```

The book includes usage guides, tool reference, RON schema docs, and
architecture diagrams. Source pages live in `book/src/` — some are
`{{#include}}` wrappers that pull content from `docs/` so that
documentation has a single source of truth.

## Claude Code Plugin

The `kb-mcp` plugin teaches Claude Code to search and read a knowledge base through kb-mcp's tools, and to set kb-mcp up for a project:

```sh
claude plugin marketplace add ttdonovan/kb-mcp
claude plugin install kb-mcp@kb-mcp
```

It adds a skill, not the server: register `kb-mcp` in each project's `.mcp.json` as above. See [Claude Code Plugins](https://ttdonovan.github.io/kb-mcp/development/plugins.html).

## Researcher Agent

A containerized agent that uses kb-mcp to research AI agent memory topics
and curate findings into the vault. Runs in a ZeroClaw container with
web search via Earl.

```sh
# Setup (web search uses DuckDuckGo — no API key needed)
cp agents/researcher/config/config.toml.ollama.example agents/researcher/config/config.toml

# Build and run
just agent-build
just agent-research-topic "HNSW vector search"
```

See [agents/researcher/README.md](agents/researcher/README.md) for full
setup and usage.

## Development

```sh
just              # List available commands
just build        # Build all crates (debug)
just release      # Build all crates (release)
just check        # cargo check --workspace
just clippy       # Lint all crates
just test         # Run all tests
just fmt          # Format all crates
just ci           # The CI gate: fmt, clippy, tests, audit, crate age
just run <args>   # Run CLI (e.g., just run list-sections)
just run-server   # Run MCP server
just docs         # Build the technical book (just book: the product book)
just docs check   # Check it: warnings, empty chapters, broken links
just claude       # Claude Code with the kb-mcp-dev plugin loaded
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for commit messages and how releases
are made.

## License

MIT
