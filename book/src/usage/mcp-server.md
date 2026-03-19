# MCP Server

kb-mcp runs as an MCP server when invoked with no arguments. It communicates
over stdio using the JSON-RPC protocol.

## Registration

Add to your project's `.mcp.json`:

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

The binary must be in your `PATH` (install via `cargo install --path .`).

## Startup

On startup, kb-mcp:

1. Loads `collections.ron` (see [Configuration](configuration.md))
2. Scans all collection directories for `.md` files
3. Builds the BM25 search index in memory
4. Starts the MCP stdio transport

Logs go to stderr. Startup typically takes <1s for ~200 documents.

## Agent Workflow

A typical agent session:

1. Call `list_sections` to see what's available
2. Call `search` to find relevant documents
3. Call `kb_context` on promising results to scan frontmatter + summary
4. Call `get_document` only on documents worth reading in full
5. Call `kb_write` to capture new knowledge (writable collections only)
6. Call `reindex` after creating new files mid-session
