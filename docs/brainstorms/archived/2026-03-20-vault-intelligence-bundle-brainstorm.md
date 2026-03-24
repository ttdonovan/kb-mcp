# Brainstorm: Vault Intelligence Bundle

**Date:** 2026-03-20
**Status:** Design complete

## What We're Building

Four lightweight features inspired by the landscape analysis that give
agents and humans better awareness of what's in the vault, keep search
results fresh, enable structured queries, and support context export.

| Feature | Inspired by | What it does |
|---------|-------------|-------------|
| `kb_digest` | hipocampus ROOT index, hmem detail levels | Generated vault summary — what's covered, what's thin, recent additions |
| `kb_query` | obsidian-web-mcp frontmatter index | Filter by frontmatter fields — tags, status, dates, sources |
| Auto-reindex on search | mnemex auto-reindex | Detect file changes at query time via modification timestamps |
| `kb_export` | mnemex pack | Export vault as a single markdown file for context seeding |

**Primary pain point:** As the vault grows, the human curator loses track
of what's covered vs what's missing. `list_sections` gives counts but not
meaning. Agents start sessions blind.

## Why These Four Together

Each feature is small individually (~50-150 lines). Together they
transform kb-mcp from "search engine" to "knowledge-aware system":

- `kb_digest` answers "what do I know?" without searching
- `kb_query` answers "show me all X" without guessing keywords
- Auto-reindex prevents stale results without manual intervention
- `kb_export` enables context sharing outside of MCP

## Key Decisions

| Decision | Choice | Alternatives Considered |
|----------|--------|------------------------|
| Vault summary approach | Generated digest from index metadata | Static ROOT.md file (hipocampus) — but we'd need to regenerate it manually |
| Frontmatter query approach | In-memory scan of `Vec<Document>` frontmatter | SQLite FTS5, dedicated index — overkill for <500 docs |
| Auto-reindex mechanism | Compare file mtime against last index timestamp | Filesystem watcher (obsidian-web-mcp) — more complex, daemon dependency |
| Export format | Single markdown with frontmatter-separated documents | JSON, SQLite dump — markdown is human-readable and LLM-friendly |

## Scope: Feature Details

### kb_digest — Vault Summary Tool

New MCP tool + CLI command. Returns a structured overview:

- Per-collection: document count, section breakdown, recent additions
  (last 7 days)
- Per-section: top topics (extracted from document titles), gap hints
  (sections with few docs)
- Vault-wide: total docs, total sections, last modified date

**Token budget:** Target ~200-500 tokens for a full vault digest. This
is the "load once at session start" context.

### kb_query — Frontmatter Structured Queries

New MCP tool + CLI command. Filter documents by frontmatter fields:

```
kb_query --tag "vector-search"
kb_query --status "draft"
kb_query --created-after "2026-03-15"
kb_query --has-source
```

Implementation: scan `Vec<Document>` frontmatter (already parsed and
stored). No new index needed — the data is in memory.

Returns: list of matching documents with path, title, tags, and the
matched frontmatter field.

### Auto-reindex on Search

Not a new tool — a behavior change to the existing `search` tool.

Before searching, check if any collection's source files have been
modified more recently than the last index timestamp. If yes, trigger
an incremental reindex (same `sync_collection` path) before returning
results.

**Cost:** One `stat()` call per collection directory on each search.
Negligible for local filesystem.

**Sidecar update:** Store last-index timestamp in the `.hashes` sidecar
file. Compare against max mtime in the collection directory.

### kb_export — Vault Export

New MCP tool + CLI command. Exports the full vault (or a single
collection) as one concatenated markdown file:

```markdown
# Vault Export: vault (2026-03-20)

## concepts/cognitive-memory-model.md
---
tags: [memory, architecture, cognitive-science]
created: 2026-03-18
---

AI agents benefit from memory systems modeled on human cognition...

---

## concepts/retrieval-strategies.md
...
```

Use case: seed a new agent's context, share vault content outside MCP,
create a snapshot for archival.

## Future Enhancements (Not in This Bundle)

These were considered but deferred — each could be a follow-up:

- **Obsolete-but-searchable markers** (hmem) — mark entries as outdated.
  Requires a frontmatter convention (`status: obsolete`) + search filter.
  Natural follow-up to `kb_query`.
- **Access-count promotion** (hmem) — rank frequently accessed docs
  higher. Requires tracking access counts, premature at 130 docs.
- **Procedural memory** (mengram) — evolving workflows. Complex, belongs
  in the Knowledge Keeper agent work.
- **Embedding model benchmarking** (mnemex) — evaluate hybrid search
  quality. Useful after hybrid search is in production use.

## Open Questions

*None — all resolved during brainstorm.*

## Resolved Questions

| Question | Resolution |
|----------|------------|
| Primary pain point? | Vault awareness gap — curator loses track of coverage |
| Which features to bundle? | All four — each is small, together they're transformative |
| Approach to vault summary? | Generated from index metadata, not a static file |
| Approach to frontmatter queries? | In-memory scan, no new index |
| Auto-reindex mechanism? | File mtime comparison, not filesystem watcher |
| Export format? | Single concatenated markdown |
