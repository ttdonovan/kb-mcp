# kb-mcp

MCP server + CLI for markdown knowledge bases.

Indexes markdown collections into a searchable BM25 index and exposes them
as [MCP](https://modelcontextprotocol.io/) tools. Configured entirely via
RON files — no hardcoded paths or project-specific values.

## Install

```sh
cargo install --path .
```

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
| `search` | BM25 full-text search with collection/section filtering |
| `get_document` | Retrieve full document content by path or title |
| `kb_context` | Token-efficient briefing (frontmatter + summary only) |
| `kb_write` | Create a note in a writable collection |
| `reindex` | Rebuild the search index from disk |

## CLI

The same binary works as a CLI when given arguments:

```sh
kb-mcp list-sections
kb-mcp search --query "rate limits"
kb-mcp search --query "bevy" --collection skills
kb-mcp get-document --path "concepts/mcp-server-pattern.md"
kb-mcp context --path "concepts/mcp-server-pattern.md"
kb-mcp write --collection notes --title "My Note" --body "Content here"
kb-mcp reindex
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

## Development

```sh
just              # List available commands
just build        # Build (debug)
just release      # Build (release)
just check        # cargo check
just clippy       # Lint
just test         # Run tests
just book-build   # Build documentation
just book-serve   # Serve docs with live reload
```

## License

MIT
