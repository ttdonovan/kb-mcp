# obsidian-web-mcp

**Source:** [github.com/jimprosser/obsidian-web-mcp](https://github.com/jimprosser/obsidian-web-mcp)
**Language:** Python | **Status:** Active (71 stars)

Secure remote MCP server for Obsidian vaults with OAuth 2.0 auth,
Cloudflare Tunnel, and atomic writes safe for Obsidian Sync.

## What It Does

obsidian-web-mcp makes your Obsidian vault accessible from anywhere —
Claude web, mobile, and desktop — via an HTTP MCP endpoint proxied
through Cloudflare Tunnel with OAuth 2.0 PKCE authentication.

## Key Features

- 9 MCP tools: read, batch read, write, frontmatter update, search,
  frontmatter search, list, move, soft-delete
- Remote access via Cloudflare Tunnel + OAuth 2.0 PKCE
- Atomic writes (write-to-temp-then-rename) safe for Obsidian Sync
- In-memory frontmatter index with filesystem watcher for auto-updates
- ripgrep for full-text search (falls back to Python if unavailable)
- Path traversal protection, safety limits (1MB/file, 20 files/batch)
- launchd plist for macOS always-on deployment

## Comparison to kb-mcp

| Aspect | obsidian-web-mcp | kb-mcp |
|--------|-----------------|--------|
| **Transport** | HTTP (remote via Cloudflare) | stdio (local) |
| **Search** | ripgrep grep (no ranking) | BM25 ranked + optional vector |
| **Vault operations** | Rich (batch read, move, delete, frontmatter) | Focused (search, read, write, context) |
| **Auth** | OAuth 2.0 PKCE | None (local only) |
| **Obsidian-specific** | Yes (Sync-safe, .trash, frontmatter index) | No (any markdown) |
| **Token efficiency** | No equivalent | kb_context (frontmatter + summary) |

**Relationship:** Different problem domains. obsidian-web-mcp solves
*remote vault access*; kb-mcp solves *effective knowledge search*. Both
serve Obsidian vaults but from opposite directions.

## Patterns Worth Adopting

- **Filesystem watcher** — auto-reindex when files change (instead of
  manual `reindex` calls)
- **Frontmatter index** — in-memory YAML index for structured queries
  beyond full-text search
- **HTTP transport** — relevant for kb-mcp's Phase 4 HTTP daemon mode
