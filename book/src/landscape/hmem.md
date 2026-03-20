# hmem

**Source:** [github.com/Bumblebiber/hmem](https://github.com/Bumblebiber/hmem)
**Language:** TypeScript | **Status:** Active (9 stars)

MCP server with 5-level lazy-loaded SQLite memory modeled after human
memory hierarchy — agents load only the detail level they need.

## What It Does

hmem stores agent memories in a hierarchical tree with 5 levels of
detail. Level 1 is a coarse summary (always loaded on agent spawn).
Levels 2-5 provide progressively more detail, fetched on demand. This
saves tokens by giving agents awareness without loading everything.

## Key Features

- 5-level hierarchical memory (coarse → verbatim)
- Tree structure with compound IDs (e.g., `L0003.2.1`)
- Markers: favorite, pinned, obsolete, irrelevant, active, secret
- Obsolete entries hidden from bulk reads but remain searchable
- Session cache with Fibonacci decay (suppresses already-seen entries)
- Access-count promotion (most-accessed entries auto-expand)
- Import/export as Markdown or SQLite
- Per-agent memory files (`.hmem`)
- Curator agent concept for periodic maintenance
- MCP over stdio (Claude Code, Gemini CLI, Cursor, Windsurf, OpenCode)

## Comparison to kb-mcp

| Aspect | hmem | kb-mcp |
|--------|------|--------|
| **Data model** | Hierarchical tree in SQLite | Flat markdown collections |
| **Search** | Tree traversal by ID (no ranking) | BM25 ranked + optional vector |
| **Token efficiency** | 5 detail levels, load only what's needed | kb_context (frontmatter + summary) |
| **Storage** | SQLite per agent | memvid-core .mv2 per collection |
| **Write pattern** | write/update/append memories | kb_write creates markdown files |
| **Maintenance** | Curator agent, Fibonacci decay, access promotion | Manual or researcher agent |

**Relationship:** Complementary. hmem excels at *structured agent working
memory* (what am I doing, what did I decide). kb-mcp excels at *reference
knowledge search* (what does the documentation say about X).

## Patterns Worth Adopting

- **Lazy-loaded detail levels** — the 5-level hierarchy is a powerful
  token-saving pattern. kb_context is a 2-level version of this (summary
  vs full document).
- **Obsolete-but-searchable** — marking entries as outdated without
  deleting them. Useful for vault knowledge that may be superseded.
- **Access-count promotion** — frequently accessed documents could be
  surfaced more prominently in search results.
- **Fibonacci decay** — suppressing recently-seen results in repeated
  queries to surface new content.
