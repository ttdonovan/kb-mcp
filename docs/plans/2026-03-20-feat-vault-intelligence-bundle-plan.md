---
title: "feat: Vault Intelligence Bundle — digest, query, auto-reindex, export"
type: feat
status: active
date: 2026-03-20
origin: docs/brainstorms/2026-03-20-vault-intelligence-bundle-brainstorm.md
---

# feat: Vault Intelligence Bundle

## Overview

Four lightweight features that transform kb-mcp from a search engine into
a knowledge-aware system. Each is small individually (~50-150 lines).
Together they solve the vault awareness gap, enable structured queries,
keep results fresh, and support context export.

(See brainstorm: `docs/brainstorms/2026-03-20-vault-intelligence-bundle-brainstorm.md`)

## Problem Statement

As the vault grows, the human curator loses track of what's covered vs
what's missing. `list_sections` gives counts but not meaning. Agents start
sessions blind. Search results can be stale after edits. There's no way
to filter by tags or dates, and no way to export vault content outside MCP.

## Proposed Solution

| New Tool | What it does | Inspired by |
|----------|-------------|-------------|
| `kb_digest` | Vault summary — coverage, gaps, recent additions | hipocampus ROOT index |
| `kb_query` | Filter by frontmatter fields (tags, status, dates) | obsidian-web-mcp |
| Auto-reindex | Detect file changes at query time | mnemex |
| `kb_export` | Export vault as single markdown file | mnemex pack |

## Technical Approach

### kb_digest — Vault Summary Tool

New MCP tool (`tools/digest.rs`) + CLI subcommand. Generates a structured
overview from the in-memory `Index`:

**Output structure:**
```json
{
  "total_documents": 15,
  "total_sections": 4,
  "collections": [
    {
      "name": "vault",
      "doc_count": 10,
      "sections": [
        {
          "name": "concepts",
          "doc_count": 3,
          "topics": ["Cognitive Memory Model", "Retrieval Strategies", "Token Efficiency"]
        }
      ],
      "recent": [
        {"path": "drafts/2026-03-19-mem0.md", "title": "Mem0", "created": "2026-03-19"}
      ]
    }
  ]
}
```

- Topics: extracted from document titles per section
- Recent: documents with `created` date in the last 7 days (from frontmatter)
- Gap hints: sections with fewer than 2 documents flagged as thin
- Target: ~200-500 tokens for a full vault digest

**Implementation:** Read from `Index.documents` and `Index.sections` —
no new data structures needed. Format via a new `format::format_digest()`.

### kb_query — Frontmatter Structured Queries

New MCP tool (`tools/query.rs`) + CLI subcommand. Filters documents by
frontmatter fields already parsed into `Document.frontmatter: HashMap`.

**Parameters:**
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    /// Filter by tag (e.g. "vector-search")
    #[serde(default)]
    pub tag: Option<String>,
    /// Filter by frontmatter status field
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by created date (YYYY-MM-DD, returns docs created on or after)
    #[serde(default)]
    pub created_after: Option<String>,
    /// Filter by collection name
    #[serde(default)]
    pub collection: Option<String>,
    /// Only return documents that have a sources field
    #[serde(default)]
    pub has_sources: bool,
}
```

**Implementation:** Linear scan of `Index.documents`, checking each
document's `frontmatter` HashMap against the query params. Returns the
same `DocumentOutput` format as `get_document` (without content body).

### Auto-reindex on Search

Not a new tool — a behavior change in `search.rs`. Before searching,
check if any collection's files have changed since the last index.

**Mechanism:**
1. Store last-index timestamp in the `.hashes` sidecar (new field)
2. On each search, stat the collection directory for max mtime
3. If mtime > last-index, trigger `sync_collection` before searching
4. Update the stored timestamp after sync

**Cost:** One `stat()` call per collection directory per search. For
local filesystem with 3 collections, this is microseconds.

**Files changed:** `store.rs` (sidecar timestamp), `search.rs` (freshness
check before query), `main.rs` (pass collections to search engine for
mtime access).

### kb_export — Vault Export

New MCP tool (`tools/export.rs`) + CLI subcommand. Concatenates all
documents in a collection (or all collections) into a single markdown
file with frontmatter separators.

**Output format:**
```markdown
# Vault Export: vault (2026-03-20)

---
## concepts/cognitive-memory-model.md
tags: [memory, architecture]
created: 2026-03-18
---

AI agents benefit from memory systems...

---
## concepts/retrieval-strategies.md
...
```

**Parameters:**
```rust
pub struct ExportParams {
    /// Collection to export (default: all)
    #[serde(default)]
    pub collection: Option<String>,
}
```

**Implementation:** Iterate `Index.documents`, read fresh from disk
(same `read_fresh` pattern as `get_document`), concatenate with
frontmatter headers.

## Acceptance Criteria

### kb_digest
- [ ] New MCP tool `kb_digest` returns vault summary JSON
- [ ] CLI subcommand `digest` produces same output
- [ ] Topics extracted from document titles per section
- [ ] Recent additions (last 7 days) included per collection
- [ ] Sections with <2 docs flagged as thin
- [ ] Output fits in ~200-500 tokens

### kb_query
- [ ] New MCP tool `kb_query` filters by tag, status, created_after, collection, has_sources
- [ ] CLI subcommand `query` with matching flags
- [ ] Returns document metadata (path, title, tags, section, collection) without body
- [ ] Multiple filters combine with AND logic
- [ ] Empty result set returns empty array (not error)

### Auto-reindex
- [ ] Search detects stale collections via file mtime comparison
- [ ] Stale collections re-synced automatically before results returned
- [ ] Fresh collections skip re-sync (no performance penalty)
- [ ] Works transparently — no new tool or parameter needed

### kb_export
- [ ] New MCP tool `kb_export` returns concatenated markdown
- [ ] CLI subcommand `export` writes to stdout
- [ ] Optional `--collection` filter
- [ ] Documents read fresh from disk (not from index)
- [ ] Frontmatter included as YAML block per document

### Quality Gates
- [ ] All 8 existing tools still work (no regression)
- [ ] `cargo clippy` clean (both default and hybrid builds)
- [ ] New tools have CLI parity (CLAUDE.md hard rule)
- [ ] Book updated with new tools in reference/tools.md

## Implementation Phases

### Phase 1: kb_digest + kb_query (new tools)

1. Create `src/tools/digest.rs` with DigestParams + format_digest
2. Create `src/tools/query.rs` with QueryParams + frontmatter filtering
3. Add both to `tools/mod.rs` router composition
4. Add `digest` and `query` CLI subcommands in `cli.rs`
5. Update server instructions in `server.rs`
6. Verify: `kb-mcp digest` and `kb-mcp query --tag "memory"` work

### Phase 2: Auto-reindex on search

1. Add last-index timestamp to `.hashes` sidecar format in `store.rs`
2. Add `check_freshness()` function comparing mtime vs stored timestamp
3. Wire freshness check into `SearchEngine::search()` before querying
4. Pass collection paths to search engine (needs access to resolve mtimes)
5. Verify: edit a vault file, search returns updated content without manual reindex

### Phase 3: kb_export

1. Create `src/tools/export.rs` with ExportParams
2. Read documents fresh from disk via `read_fresh` pattern
3. Concatenate with frontmatter headers
4. Add `export` CLI subcommand
5. Verify: `kb-mcp export --collection vault` produces valid markdown

### Phase 4: Docs + polish

1. Update `book/src/reference/tools.md` with all 3 new tools
2. Update `docs/ROADMAP.md` — add vault intelligence to completed
3. Run `cargo clippy` on both default and hybrid builds
4. Update `docs/ARCHITECTURE.md` if needed

## Sources & References

### Origin

- **Brainstorm:** [docs/brainstorms/2026-03-20-vault-intelligence-bundle-brainstorm.md](../brainstorms/2026-03-20-vault-intelligence-bundle-brainstorm.md) — four features bundled, in-memory approaches, mtime for auto-reindex

### Landscape Inspiration

- **hipocampus** — ROOT.md topic index for O(1) vault awareness → `kb_digest`
- **obsidian-web-mcp** — frontmatter index + structured queries → `kb_query`
- **mnemex** — auto-reindex on search + pack/export → auto-reindex + `kb_export`
- **hmem** — lazy-loaded detail levels → validates kb_context approach

### Key Files

- `src/index.rs` — `Index` struct with documents + sections + content_hashes
- `src/types.rs` — `Document.frontmatter: HashMap` (already parsed)
- `src/format.rs` — output formatting patterns to follow
- `src/tools/mod.rs` — router composition (add 3 new tools)
- `src/server.rs` — `read_fresh()` for disk reads
- `src/store.rs` — `.hashes` sidecar (add timestamp)
- `src/search.rs` — freshness check injection point
