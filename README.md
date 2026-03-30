# kb-mcp

MCP server + CLI for markdown knowledge bases.

[Read the Book](https://ttdonovan.github.io/kb-mcp/) | [GitHub](https://github.com/ttdonovan/kb-mcp)

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

Collection paths resolve relative to the config file's location.

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

Project documentation is built with [mdBook](https://rust-lang.github.io/mdBook/)
and uses [Mermaid](https://mermaid.js.org/) for architecture diagrams.

### Prerequisites

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
just run <args>   # Run CLI (e.g., just run list-sections)
just run-server   # Run MCP server
just book-build   # Build documentation
just book-serve   # Serve docs with live reload
```

## License

MIT
