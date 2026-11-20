---
name: kb-mcp
description: Find and read notes in a markdown knowledge base through kb-mcp's MCP tools (search, kb_context, get_document, kb_write and the rest), and set kb-mcp up for a project - install it, write collections.ron, register the server. Use when kb-mcp's tools are available in the session, or when asked to set up kb-mcp or a searchable markdown knowledge base for agents.
---

# kb-mcp

kb-mcp indexes markdown folders ("collections", defined in `collections.ron`) into a BM25 search index and serves them over MCP. The `kb` CLI has a command for every tool, with the same JSON output. The full docs are the book: https://ttdonovan.github.io/kb-mcp/

## Using the tools

Read cheaply first, in full only what matters:

1. `kb_digest` or `list_sections` to see what the knowledge base covers, once per task.
2. `search` with a few keywords. Narrow with `collection` or `scope` (a section prefix). It is BM25: use the words a note would contain, and try synonyms if the first query misses.
3. `kb_context` on the promising results: frontmatter and the first paragraph, a fraction of the tokens.
4. `get_document` only for the notes you need whole. It reads from disk, so it is always current.

- `kb_query` filters by frontmatter (tag, status, created date, has sources) without a query.
- `kb_health` lists quality problems: missing dates or tags, stale notes, stubs, orphans, broken wiki-links.
- `kb_export` concatenates a collection into one markdown document, capped by `max_documents`.
- `kb_write` creates a note, only in a collection with `writable: true`; others return an error. A file name already in use gets a numeric suffix (`-2`, `-3`).
- The server notices added or removed files on its own. After editing existing files, call `reindex`.

Tool parameters: https://ttdonovan.github.io/kb-mcp/reference/tools.html

## Setting it up for a project

1. Install `kb` and `kb-mcp`. Prebuilt binaries (Linux x86_64/arm64, macOS arm64/x86_64) with `SHA256SUMS` and build provenance are on each release: https://github.com/ttdonovan/kb-mcp/releases. Verify a download with `gh attestation verify <file> --repo ttdonovan/kb-mcp`. Or build from a tag: `cargo install --locked --git https://github.com/ttdonovan/kb-mcp --tag vX.Y.Z kb-cli kb-mcp-server`.
2. Write `collections.ron` at the project root. Each collection has a `name`, a `path` (relative to the config file), a `description`, `writable` and optional `sections` (a directory prefix with a description). Start from the example: https://github.com/ttdonovan/kb-mcp/blob/main/collections.example.ron. Every field: https://ttdonovan.github.io/kb-mcp/reference/ron-schema.html
3. Register the server in the project's `.mcp.json`:

   ```json
   { "mcpServers": { "kb": { "command": "kb-mcp", "args": [] } } }
   ```

4. Check it with the CLI from the project root: `kb list-sections`, then `kb search --query "<words>"`.

kb-mcp finds its config by `--config`, then `KB_MCP_CONFIG`, then `./collections.ron`, then `~/.config/kb-mcp/collections.ron`. To serve one knowledge base to several projects, set `KB_MCP_CONFIG` in the `env` of each `.mcp.json` entry.

## Gotchas

- The index lives in `cache_dir` (default `kb-mcp` in the user's cache folder, such as `~/.cache/kb-mcp` on Linux or `~/Library/Caches/kb-mcp` on macOS; `KB_MCP_CACHE_DIR` overrides it). Deleting it is safe: kb-mcp rebuilds it on the next start.
- By default an index file is named by a hash of the collection's absolute path, so a moved project builds a new index. `index_names: Name` keeps the name stable, but then `cache_dir` must belong to this config alone.
- Several servers can share a cache dir; one that finds another's index replaced reopens it.
- `collections.ron` is often gitignored, as it holds local paths; commit an example beside it.
- Semantic search needs a build with the `hybrid` feature and a local model; the release binaries are BM25 only. See https://ttdonovan.github.io/kb-mcp/usage/hybrid-search.html
