---
title: "feat: kb_health — vault health diagnostics tool"
type: feat
status: completed
date: 2026-03-22
origin: docs/brainstorms/2026-03-21-ori-mnemos-learnings-brainstorm.md
---

# feat: kb_health — Vault Health Diagnostics

## Overview

New MCP tool + CLI subcommand that checks document quality and hygiene
across collections. Flags missing frontmatter, stale content, stub
documents, orphaned notes, and broken wiki-links.

Inspired by Ori-Mnemos `ori_health`. This is the one Ori pattern that
cleanly fits kb-mcp's identity as a knowledge base server.
(See brainstorm: `docs/brainstorms/2026-03-21-ori-mnemos-learnings-brainstorm.md`)

## Problem Statement

As vaults grow, quality degrades silently. Documents accumulate without
tags, dates go stale, stub placeholders are forgotten, and wiki-links
break. `kb_digest` shows coverage (sections, topics, recent additions)
but not quality. Agents and curators need a tool that surfaces hygiene
issues so they can be fixed.

## Proposed Solution

A single `kb_health` tool with two categories of checks:

**Frontmatter & content checks** (from existing Index data):
- Documents missing `created` date
- Documents missing `updated` date
- Documents with no tags
- Stale documents (last date older than threshold)
- Stub documents (word count below threshold)

**Link analysis** (regex scan of in-memory `doc.body`):
- Orphan documents (zero inbound wiki-links)
- Broken wiki-links (target not found)

## Technical Approach

### Parameters

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HealthParams {
    /// Filter to a specific collection
    #[serde(default)]
    pub collection: Option<String>,
    /// Days threshold for staleness (default: 90)
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    /// Minimum word count — below this is a stub (default: 50)
    #[serde(default = "default_min_words")]
    pub min_words: u32,
}
```

### Output Structure

```json
{
  "total_documents_checked": 48,
  "total_issues": 12,
  "collections": [
    {
      "name": "vault",
      "doc_count": 26,
      "issues": 8,
      "missing_created": [{ "path": "...", "title": "..." }],
      "missing_updated": [{ "path": "...", "title": "..." }],
      "no_tags": [{ "path": "...", "title": "..." }],
      "stale": [{ "path": "...", "title": "...", "last_date": "2025-06-01" }],
      "stubs": [{ "path": "...", "title": "...", "word_count": 12 }],
      "orphans": [{ "path": "...", "title": "..." }],
      "broken_links": [
        { "source": "concepts/mcp.md", "target": "Nonexistent Doc", "raw": "[[Nonexistent Doc]]" }
      ]
    }
  ]
}
```

### Key Design Decisions

**Staleness precedence:** Use `updated` if present, fall back to `created`.
Documents with neither date are excluded from staleness checks (they
already appear in the `missing_created` / `missing_updated` lists).

**Wiki-link resolution:** Match `[[target]]` against `doc.title`
(case-insensitive) with fallback to `doc.path` (case-insensitive,
`.md` extension optional). Strip alias text after `|`. Strip heading
anchors after `#`.

**Wiki-link scope:** Global across all collections. Matches how Obsidian
vaults work — a link in collection A can reference a doc in collection B.

**Orphan detection:** A document is orphaned if no other document links
to it via wiki-link. If the vault has zero wiki-links total, orphan
detection is skipped (every doc would be flagged, which is unhelpful).

**Word count:** Naive `body.split_whitespace().count()`. Simple, fast,
good enough for stub detection.

**Invalid collection name:** Return an error with available collection
names, consistent with `kb_write`.

**No output cap:** All flagged documents are returned. Agents can
truncate if needed.

### Implementation

**New file: `src/tools/health.rs`**

Params struct + router + tool method. Calls `self.auto_reindex_stale_collections().await`
before reading the index (learned from vault intelligence bundle — cross-cutting
concern must apply to all index-reading tools).

Health check logic:

1. Filter documents by collection (if specified)
2. Run frontmatter checks: iterate `doc.frontmatter` for `created`,
   `updated`, `tags`. Parse dates via `NaiveDate::parse_from_str`
   with `%Y-%m-%d` format (same as `format_digest`).
3. Run word count: `doc.body.split_whitespace().count()`
4. Build wiki-link graph: regex scan all docs for `\[\[([^\]]+)\]\]`,
   resolve targets, build `HashMap<String, Vec<String>>` of
   inbound links per doc.
5. Detect orphans (zero inbound) and broken links (target not resolved).
6. Delegate to `format::format_health()` for JSON output.

**Format layer: `src/format.rs`**

Add `HealthOutput`, `HealthCollectionOutput`, `HealthDocRef`,
`HealthStaleRef`, `HealthStubRef`, `HealthBrokenLinkRef` structs.
Add `format_health()` function.

**CLI: `src/cli.rs`**

Add `Health` variant with `--collection`, `--stale-days`, `--min-words`
flags. Handler calls `format::format_health()` with same logic.

**Router: `src/tools/mod.rs`**

Add `pub(crate) mod health;` and `+ health::router()`.

**Server instructions: `src/server.rs`**

Update instructions to mention `kb_health` for document quality checks,
distinguishing from `kb_digest` (coverage overview).

## Acceptance Criteria

### Frontmatter Checks
- [x] Flags documents missing `created` frontmatter field
- [x] Flags documents missing `updated` frontmatter field
- [x] Flags documents with empty tags list
- [x] Flags stale documents (last date > `stale_days` threshold)
- [x] Staleness uses `updated` with `created` fallback
- [x] Documents missing both dates excluded from staleness (not double-flagged)

### Content Checks
- [x] Flags stub documents (word count < `min_words` threshold)
- [x] Word count uses `split_whitespace().count()` on stripped body

### Link Analysis
- [x] Extracts wiki-links via `\[\[([^\]]+)\]\]` regex
- [x] Strips alias text after `|` and heading anchors after `#`
- [x] Resolves targets by title (case-insensitive) then path (case-insensitive)
- [x] Flags orphan documents (zero inbound wiki-links)
- [x] Flags broken wiki-links (target not resolved to any document)
- [x] Skips orphan detection when vault has zero wiki-links total
- [x] Link resolution is global across all collections

### Parameters & Errors
- [x] `collection` filter works correctly
- [x] Invalid collection name returns error with available names
- [x] `stale_days` defaults to 90, configurable
- [x] `min_words` defaults to 50, configurable

### Quality Gates
- [x] New MCP tool `kb_health` returns health report JSON
- [x] CLI subcommand `health` with matching flags
- [x] Format function in `format.rs` (not inline in tool)
- [x] Auto-reindex guard before index read
- [x] `cargo clippy` clean
- [x] All existing tools still work (no regression)
- [x] Book updated with kb_health in reference/tools.md
- [x] Server instructions updated

## Implementation Phases

### Phase 1: Frontmatter + content checks (~80-100 lines)

1. Create `src/tools/health.rs` with HealthParams
2. Add frontmatter checks (missing dates, no tags, staleness)
3. Add word count stub detection
4. Add format structs + `format_health()` to `format.rs`
5. Wire into `tools/mod.rs` router
6. Add CLI `Health` subcommand
7. Verify: `kb-mcp health` works

### Phase 2: Link analysis (~50-80 lines)

1. Add wiki-link regex extraction function
2. Build link graph from all documents
3. Add orphan detection
4. Add broken link detection
5. Integrate into `format_health()` output
6. Verify: `kb-mcp health` shows orphans and broken links

### Phase 3: Polish

1. Update server instructions in `server.rs`
2. Update `book/src/reference/tools.md`
3. Run `cargo clippy` on both default and hybrid builds
4. Verify all 10 existing tools still work

## Sources & References

### Origin

- **Brainstorm:** [docs/brainstorms/2026-03-21-ori-mnemos-learnings-brainstorm.md](../../brainstorms/archived/2026-03-21-ori-mnemos-learnings-brainstorm.md) — vault health diagnostics classified as the one Ori-Mnemos pattern that fits kb-mcp's identity

### Landscape Inspiration

- **Ori-Mnemos** — `ori_health` tool checks index freshness, orphan notes, dangling wiki-links, backlink counts, vault metrics
- **hipocampus** — ROOT.md topic index for vault awareness (already adopted as `kb_digest`)

### Institutional Learnings

- **Vault Intelligence Bundle** ([docs/solutions/feature-patterns/vault-intelligence-bundle-learnings.md](../solutions/feature-patterns/vault-intelligence-bundle-learnings.md)) — format layer separation, auto-reindex guard on all index-reading tools, filtered views must use filtered counts, invalid input is an error

### Key Files

- `src/tools/digest.rs` — closest sibling tool (same Index-reading pattern)
- `src/format.rs` — output formatting (add HealthOutput structs here)
- `src/tools/mod.rs` — router composition (add health::router())
- `src/index.rs` — Index struct with documents + sections
- `src/types.rs` — Document struct with frontmatter HashMap
- `src/server.rs` — auto_reindex_stale_collections + server instructions
