# kb-mcp

MCP server + CLI for markdown knowledge bases.

Indexes markdown collections into a searchable BM25 index and exposes them
as [MCP](https://modelcontextprotocol.io/) tools. Configured entirely via
RON files — no hardcoded paths or project-specific values.

## Features

- **6 MCP tools** — list, search, get, context briefing, write, reindex
- **CLI parity** — every MCP tool works as a CLI subcommand
- **RON configuration** — typed, Rust-native config with comments
- **Collection model** — multiple collections with sections, descriptions, and writable flags
- **Token-efficient** — `kb_context` returns frontmatter + summary without the full body
- **Write-back** — `kb_write` creates notes with proper frontmatter in writable collections

## Quick Start

```sh
# Install
cargo install --path .

# Create config
cp collections.example.ron collections.ron
# Edit paths to point at your markdown directories

# Use as CLI
kb-mcp list-sections
kb-mcp search --query "your query"

# Use as MCP server (register in .mcp.json)
```

## How It Works

1. On startup, reads `collections.ron` to discover markdown collections
2. Scans each collection directory for `.md` files
3. Parses frontmatter (YAML) and body content
4. Builds an in-memory BM25 search index (Tantivy)
5. Serves queries via MCP stdio transport or CLI
