---
tags: [patterns, obsidian, workflow, skills]
created: 2026-03-18
updated: 2026-03-18
sources:
  - https://github.com/kepano/obsidian-skills
---

# Obsidian + kb-mcp Workflow

kb-mcp indexes markdown collections. Obsidian is a markdown-native vault
editor. Together they form a complete knowledge management workflow where
humans curate in Obsidian and agents query/write via kb-mcp.

## The Loop

```
Human writes/edits in Obsidian
  → kb-mcp indexes the vault on startup (or via reindex)
  → Agent searches and reads via MCP tools
  → Agent writes new knowledge via kb_write
  → Human reviews new entries in Obsidian
  → cycle continues
```

The vault is the single source of truth. kb-mcp is the agent interface
to it. Neither tool replaces the other.

## Claude Code Skills

Three Obsidian skills from [kepano/obsidian-skills](https://github.com/kepano/obsidian-skills)
(by Steph Ango, Obsidian CEO) are included in `.claude/skills/` to help
agents create well-formed vault content:

### obsidian-markdown

Covers Obsidian Flavored Markdown — wikilinks, embeds, callouts,
properties (frontmatter), and comments. Use this when creating vault
notes so they render correctly in both Obsidian and kb-mcp.

**Key points for kb-mcp compatibility:**
- Frontmatter must be valid YAML between `---` delimiters — kb-mcp
  parses this for `kb_context` output
- Wikilinks (`[[Note]]`) work in Obsidian but not in mdBook — use
  standard links in content that appears in both
- Tags in frontmatter (`tags: [memory, patterns]`) are indexed by
  kb-mcp's search engine

### obsidian-bases

Creates `.base` files for database-like views of vault notes — tables,
cards, lists filtered by tags, folders, or properties.

**How this complements kb-mcp:**
- Bases provide visual dashboards in Obsidian (e.g., "all notes tagged
  `research` sorted by date")
- kb-mcp provides programmatic search for the same content
- Use Bases for human navigation, kb-mcp for agent navigation

### obsidian-cli

Interacts with a running Obsidian instance from the command line —
create notes, search, manage tasks, read properties.

**When to use alongside kb-mcp:**
- `obsidian-cli` requires Obsidian to be running — use it for live
  vault manipulation during active editing sessions
- `kb-mcp` works without Obsidian running — use it for agent workflows,
  CI/CD, or headless environments
- Both can coexist: cli for real-time interaction, MCP for search/retrieval

## Vault Conventions for kb-mcp

For vaults that will be indexed by kb-mcp:

1. **Always include frontmatter** — tags, created, updated at minimum.
   `kb_context` returns these fields for token-efficient briefings.

2. **Use the first H1 as the title** — kb-mcp extracts it. If missing,
   it falls back to the filename.

3. **Organize by directory** — the first subdirectory becomes the section
   in kb-mcp. `vault/concepts/foo.md` → section "concepts".

4. **Define sections in RON config** — give directory prefixes
   human-readable descriptions that appear in `list_sections` output.

5. **Keep one topic per file** — kb-mcp search ranks individual documents.
   A single file covering 10 topics will match many queries with low
   relevance. Split it.

## Example: Adding Knowledge

In Obsidian, create a new note:

```markdown
---
tags: [memory, vector-search, research]
created: 2026-03-18
updated: 2026-03-18
sources:
  - https://arxiv.org/abs/2304.03442
---

# HNSW Index Performance at Scale

Testing showed that HNSW maintains sub-millisecond recall up to...
```

This is immediately searchable via kb-mcp after a `reindex`:

```sh
kb-mcp search --query "HNSW performance" --collection vault
kb-mcp context --path "research/hnsw-performance.md"
```

Or via `kb_write` for agent-created knowledge:

```sh
kb-mcp write \
  --collection vault \
  --title "HNSW Index Performance at Scale" \
  --tags "memory,vector-search,research" \
  --body "Testing showed that HNSW maintains..."
```

Both paths produce the same result: a well-formed vault note that's
searchable by agents and browsable in Obsidian.
